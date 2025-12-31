use crate::engine::PopStarEngine;
use rand::seq::IndexedRandom;
use std::f64;

/// MCTS 节点结构
struct Node {
    engine: PopStarEngine,
    parent: Option<usize>,          // 父节点索引 (在 Arena 中的索引)
    children: Vec<usize>,           // 子节点索引列表
    action: Option<(usize, usize)>, // 到达此节点的动作
    visits: u32,
    value: f64,
    untried_actions: Vec<((usize, usize), Vec<(usize, usize)>)>, // (action, group)
}

impl Node {
    fn new(engine: PopStarEngine, parent: Option<usize>, action: Option<(usize, usize)>) -> Self {
        let untried = Self::get_all_moves(&engine);
        Node {
            engine,
            parent,
            children: Vec::new(),
            action,
            visits: 0,
            value: 0.0,
            untried_actions: untried,
        }
    }

    /// 获取当前局面的所有合法动作及其连通组
    /// 为了优化，我们同时存储 group 数据，避免 expand 时再次 BFS
    fn get_all_moves(engine: &PopStarEngine) -> Vec<((usize, usize), Vec<(usize, usize)>)> {
        let mut moves = Vec::new();
        let mut visited = [false; 100];

        for r in 0..10 {
            for c in 0..10 {
                let idx = r * 10 + c;
                if engine.board[idx] != -1 && !visited[idx] {
                    let group = engine.get_connected_group(r, c);
                    // 标记 visited
                    for &(gr, gc) in &group {
                        visited[gr * 10 + gc] = true;
                    }

                    if group.len() >= 2 {
                        moves.push(((r, c), group));
                    }
                }
            }
        }
        moves
    }
}

/// MCTS 求解器
pub struct PopStarSolver {
    nodes: Vec<Node>, // 使用 Arena 方式存储节点，避免自引用生命周期地狱
    root_idx: usize,
}

impl PopStarSolver {
    pub fn new(engine: PopStarEngine) -> Self {
        let root = Node::new(engine, None, None);
        PopStarSolver {
            nodes: vec![root],
            root_idx: 0,
        }
    }

    /// 执行 MCTS 搜索
    /// # 参数
    /// - `iterations`: 模拟次数
    /// # 返回
    /// (最佳动作, 最大搜索得分, 最佳路径)
    pub fn solve(
        &mut self,
        iterations: usize,
    ) -> (Option<(usize, usize)>, i32, Vec<(usize, usize)>) {
        let mut max_score_found = 0;
        let mut best_path_found = Vec::new();

        for _ in 0..iterations {
            let mut node_idx = self.root_idx;

            // 1. 选择 (Select): 选择直到叶子节点或还有未尝试动作的节点
            loop {
                let node = &self.nodes[node_idx];
                if !node.untried_actions.is_empty() {
                    break;
                }
                if node.children.is_empty() {
                    break;
                }
                // UCB 选择
                node_idx = self.best_ucb_child(node_idx);
            }

            // 2. 扩展 (Expand): 如果有未尝试动作，展开一个新节点
            let mut curr_idx = node_idx;
            // 只是为了借用检查，这里稍微绕一下
            let has_untried = !self.nodes[curr_idx].untried_actions.is_empty();

            if has_untried {
                let ((r, c), group) = self.nodes[curr_idx].untried_actions.pop().unwrap();
                let mut next_engine = self.nodes[curr_idx].engine.clone();
                next_engine.eliminate(r, c, Some(group));

                let new_node = Node::new(next_engine, Some(curr_idx), Some((r, c)));
                let new_idx = self.nodes.len();
                self.nodes.push(new_node);
                self.nodes[curr_idx].children.push(new_idx);
                curr_idx = new_idx;
            }

            // 3. 模拟 (Simulate): 随机模拟直到结束
            // 获取当前节点状态的拷贝进行模拟
            let mut sim_engine = self.nodes[curr_idx].engine.clone();
            let (sim_delta, context_path) = self.simulate(&mut sim_engine);

            // 检查是否发现新的历史最高分 (当前节点得分 + 模拟增量 + 结束奖励)
            // sim_delta 已经包含了模拟过程中的得分 + 结束奖励
            let current_total = self.nodes[curr_idx].engine.total_score + sim_delta;

            if current_total > max_score_found {
                max_score_found = current_total;
                // 重建路径: root -> curr -> sim_path
                let mut path = self.reconstruct_path(curr_idx);
                path.extend(context_path);
                best_path_found = path;
            }

            // 4. 回溯 (Backpropagate): 回溯更新
            let score_delta = (current_total - self.nodes[self.root_idx].engine.total_score) as f64;
            self.backpropagate(curr_idx, score_delta);
        }

        // 获取最终推荐：优先使用搜索到的全局最佳路径
        let (best_action, final_path) = self.get_final_recommendation(&best_path_found);

        (best_action, max_score_found, final_path)
    }

    /// 使用 UCB 公式选择最佳子节点
    fn best_ucb_child(&self, parent_idx: usize) -> usize {
        let parent = &self.nodes[parent_idx];
        let log_n = (parent.visits as f64).ln();

        // 调整探索系数以适应游戏得分规模 (平均得分为几百到几千)
        // 使用较大的 C 值 (如 100.0) 以鼓励在早期探索
        let c = 100.0;

        *parent
            .children
            .iter()
            .max_by(|&&a_idx, &&b_idx| {
                let a = &self.nodes[a_idx];
                let b = &self.nodes[b_idx];

                let ucb_a = a.value / (a.visits as f64) + c * (log_n / a.visits as f64).sqrt();
                let ucb_b = b.value / (b.visits as f64) + c * (log_n / b.visits as f64).sqrt();
                ucb_a.partial_cmp(&ucb_b).unwrap()
            })
            .unwrap()
    }

    /// 随机模拟
    /// 返回 (模拟获得的额外分数 + 结束奖励, 模拟的路径)
    fn simulate(&self, engine: &mut PopStarEngine) -> (i32, Vec<(usize, usize)>) {
        let initial_score = engine.total_score;
        let mut path = Vec::new();
        let mut rng = rand::rng();

        loop {
            let moves = Node::get_all_moves(engine);
            if moves.is_empty() {
                break;
            }

            // 权重选择: 连消权重。
            // 越大的块被选中的概率越高，这有助于更早发现高分路径
            // 使用 (n^2) 作为权重，模拟真实游戏中对大块的偏好
            let chosen = moves
                .choose_weighted(&mut rng, |m| {
                    let n = m.1.len() as f32;
                    n * n
                })
                .unwrap()
                .clone();

            let ((r, c), group) = chosen;
            engine.eliminate(r, c, Some(group));
            path.push((r, c));
        }

        let end_bonus = engine.calculate_end_bonus();
        let total = engine.total_score + end_bonus;
        (total - initial_score, path)
    }

    fn backpropagate(&mut self, leaf_idx: usize, result: f64) {
        let mut curr = Some(leaf_idx);
        while let Some(idx) = curr {
            let node = &mut self.nodes[idx];
            node.visits += 1;
            node.value += result;
            curr = node.parent;
        }
    }

    fn reconstruct_path(&self, node_idx: usize) -> Vec<(usize, usize)> {
        let mut path = Vec::new();
        let mut curr = Some(node_idx);
        while let Some(idx) = curr {
            if let Some(act) = self.nodes[idx].action {
                path.push(act);
            }
            curr = self.nodes[idx].parent;
        }
        path.reverse();
        path
    }
}

impl PopStarSolver {
    /// 获取最终推荐。
    /// 对于单人益智游戏，首选全盘搜索到的历史最佳路径的第一步。
    fn get_final_recommendation(
        &self,
        best_path: &[(usize, usize)],
    ) -> (Option<(usize, usize)>, Vec<(usize, usize)>) {
        let root = &self.nodes[self.root_idx];
        if root.children.is_empty() {
            return (None, Vec::new());
        }

        // 策略: 如果有历史最佳路径且非空，直接返回。
        // 这是最符合用户"追求最高分"直觉的选择。
        if !best_path.is_empty() {
            return (Some(best_path[0]), best_path.to_vec());
        }

        // 降级策略: 访问次数最多的子节点
        let best_child_idx = *root
            .children
            .iter()
            .max_by_key(|&&idx| self.nodes[idx].visits)
            .unwrap();
        let best_action = self.nodes[best_child_idx].action;
        let best_path_for_child = self.reconstruct_path(best_child_idx);

        (best_action, best_path_for_child)
    }
}
