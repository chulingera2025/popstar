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
            let (_sim_score, context_path) = self.simulate(&mut sim_engine);

            // 检查是否发现新的历史最高分 (引擎当前分 + 模拟增量 + 结束奖励)
            // 注意: sim_score 已经包含了 simulate 过程中的得分 + 结束奖励
            // 但我们需要加上到达 curr_node 之前的得分
            let current_total = sim_engine.total_score; // simulate 会直接修改这个 clone 的 engine

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

        // 返回访问次数最多的子节点动作
        let root = &self.nodes[self.root_idx];
        if root.children.is_empty() {
            return (None, max_score_found, best_path_found);
        }

        let best_child_idx = *root
            .children
            .iter()
            .max_by_key(|&&idx| self.nodes[idx].visits)
            .unwrap();
        let best_action = self.nodes[best_child_idx].action;

        (best_action, max_score_found, best_path_found)
    }

    /// 使用 UCB 公式选择最佳子节点
    fn best_ucb_child(&self, parent_idx: usize) -> usize {
        let parent = &self.nodes[parent_idx];
        let log_n = (parent.visits as f64).ln(); // precompute log(N)

        *parent
            .children
            .iter()
            .max_by(|&&a_idx, &&b_idx| {
                let a = &self.nodes[a_idx];
                let b = &self.nodes[b_idx];

                let ucb_a = a.value / (a.visits as f64) + (2.0 * log_n / a.visits as f64).sqrt();
                let ucb_b = b.value / (b.visits as f64) + (2.0 * log_n / b.visits as f64).sqrt();
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
            // 快速获取所有合法移动，不再全盘扫描
            // 这里我们优化一下：与其每次生成所有 move，不如只生成 move 的坐标
            // 为了最快速度，我们还是得得一次性找出来。
            // 优化点：Node::get_all_moves 已经很快了，但我们不需要存 copy 的 group，只需要坐标和group本身

            let moves = Node::get_all_moves(engine);
            if moves.is_empty() {
                break;
            }

            // 随机选择
            let ((r, c), group) = moves.choose(&mut rng).unwrap().clone();
            engine.eliminate(r, c, Some(group));
            path.push((r, c));
        }

        let end_bonus = engine.calculate_end_bonus();
        let total = engine.total_score + end_bonus; // 注意 engine.total_score 已经变了
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
