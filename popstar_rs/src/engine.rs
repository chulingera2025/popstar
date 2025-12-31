use rand::Rng;

/// 游戏常量定义
pub const WIDTH: usize = 10;
pub const HEIGHT: usize = 10;
pub const BOARD_SIZE: usize = WIDTH * HEIGHT;

/// PopStar 游戏核心引擎
///
/// 负责维护游戏棋盘状态、执行消除逻辑、处理重力下落和列合并。
/// 棋盘使用一维数组 `[i8; 100]` 表示，以优化缓存性能。
/// 值定义:
/// - `-1`: 空位
/// - `0-4`: 五种颜色
#[derive(Clone, Debug)]
pub struct PopStarEngine {
    pub board: [i8; BOARD_SIZE],
    pub score: i32,
    pub total_score: i32,
}

impl PopStarEngine {
    /// 创建一个新的游戏引擎实例
    ///
    /// # 参数
    /// - `board`: 可选的初始棋盘数据。如果为 `None`，则随机生成。
    pub fn new(board: Option<Vec<i8>>) -> Self {
        let mut engine = PopStarEngine {
            board: [0; BOARD_SIZE],
            score: 0,
            total_score: 0,
        };

        if let Some(b) = board {
            if b.len() == BOARD_SIZE {
                engine.board.copy_from_slice(&b);
            } else {
                panic!("Invalid board size");
            }
        } else {
            // 随机初始化棋盘
            let mut rng = rand::rng();
            for i in 0..BOARD_SIZE {
                engine.board[i] = rng.random_range(0..5);
            }
        }
        engine
    }

    /// 获取指定坐标 (row, col) 在一维数组中的索引
    #[inline(always)]
    fn idx(&self, r: usize, c: usize) -> usize {
        r * WIDTH + c
    }

    /// 获取 (r, c) 坐标所在的同色连通区域
    ///
    /// # 返回值
    /// 返回一个包含所有连通块坐标的 `Vec<(usize, usize)>`。
    /// 如果该位置为空或越界，返回空集合。
    pub fn get_connected_group(&self, r: usize, c: usize) -> Vec<(usize, usize)> {
        let color = self.board[self.idx(r, c)];
        if color == -1 {
            return Vec::new(); // 空位没有连通区域
        }

        let mut group = Vec::new();
        let mut stack = vec![(r, c)];
        let mut visited = vec![false; BOARD_SIZE];

        visited[self.idx(r, c)] = true;
        group.push((r, c));

        while let Some((cr, cc)) = stack.pop() {
            // 检查上下左右四个方向
            let directions = [(0, 1), (0, -1), (1, 0), (-1, 0)];
            for (dr, dc) in directions {
                let nr = cr as isize + dr;
                let nc = cc as isize + dc;

                if nr >= 0 && nr < HEIGHT as isize && nc >= 0 && nc < WIDTH as isize {
                    let nr = nr as usize;
                    let nc = nc as usize;
                    let idx = self.idx(nr, nc);

                    if !visited[idx] && self.board[idx] == color {
                        visited[idx] = true;
                        stack.push((nr, nc));
                        group.push((nr, nc));
                    }
                }
            }
        }
        group
    }

    /// 执行消除操作
    ///
    /// # 参数
    /// - `r`, `c`: 点击的坐标
    /// - `known_group`: 可选的预计算连通组，用于优化性能避免重复搜索
    ///
    /// # 返回值
    /// 本次消除获得的得分。如果无法消除（连通数<2或空位），返回 0。
    pub fn eliminate(
        &mut self,
        r: usize,
        c: usize,
        known_group: Option<Vec<(usize, usize)>>,
    ) -> i32 {
        if self.board[self.idx(r, c)] == -1 {
            return 0;
        }

        // 使用预计算组或重新搜索
        let group = match known_group {
            Some(g) => g,
            None => self.get_connected_group(r, c),
        };

        let n = group.len();
        if n < 2 {
            return 0;
        }

        // 1. 计算得分: n^2 * 5
        let move_score = (n * n * 5) as i32;
        self.score += move_score;
        self.total_score += move_score;

        // 2. 标记消除 (设为 -1)
        for (gr, gc) in group {
            let idx = self.idx(gr, gc);
            self.board[idx] = -1;
        }

        // 3. 应用重力
        self.apply_gravity();

        // 4. 列合并
        self.apply_column_shift();

        move_score
    }

    /// 应用重力下落逻辑
    ///
    /// 方块悬空时会自动掉落填补下方空位。
    fn apply_gravity(&mut self) {
        for c in 0..WIDTH {
            let mut write_idx = HEIGHT - 1; // 从底部开始写的指针
            // 从底部向上扫描
            for r in (0..HEIGHT).rev() {
                let idx = self.idx(r, c);
                if self.board[idx] != -1 {
                    // 如果当前位置不是空位
                    if r != write_idx {
                        // 移动到 write_idx 位置
                        self.board[self.idx(write_idx, c)] = self.board[idx];
                        self.board[idx] = -1; // 原位置置空
                    }
                    if write_idx > 0 {
                        write_idx -= 1;
                    }
                }
            }
            // 剩余上方的区域已经是 -1 了，不需要额外填充，因为我们是将非 -1 的值"搬运"下去
            // 但如果上面的逻辑没清除原位置（即 r == write_idx 没动），则不需要清理
            // 这里为了严谨，其实上面的 swap 逻辑已经涵盖了。
            // 修正：上面的逻辑是从下往上找非空，依次填入 write_idx。
            // write_idx 及其上方的所有位置最后都应该是 -1。
            // 例如 [1, -1, 2] (顶->底)
            // r=2(2): write_idx=2, board[2]=2. write_idx-- -> 1
            // r=1(-1): skip
            // r=0(1): write_idx=1, board[1]=1, board[0]=-1. wrize_idx-- -> 0
            // 结果 [ -1, 1, 2 ] 正确。
            // 但如果本身是满的 [1, 2, 3]
            // r=2(3): w=2, same, w->1
            // r=1(2): w=1, same, w->0
            // r=0(1): w=0, same.
            // 所以要加 r != write_idx 判断来避免自我赋值和清空。
        }
    }

    /// 应用列左移逻辑
    ///
    /// 当某一列完全为空时，右侧的列整体向左移动填补。
    fn apply_column_shift(&mut self) {
        let mut write_col = 0;
        for c in 0..WIDTH {
            // 检查当前列是否全空
            let mut is_empty = true;
            for r in 0..HEIGHT {
                if self.board[self.idx(r, c)] != -1 {
                    is_empty = false;
                    break;
                }
            }

            if !is_empty {
                if c != write_col {
                    // 搬运整列
                    for r in 0..HEIGHT {
                        self.board[self.idx(r, write_col)] = self.board[self.idx(r, c)];
                        self.board[self.idx(r, c)] = -1; // 原列置空
                    }
                }
                write_col += 1;
            }
        }
    }

    /// 计算最终剩余奖励
    pub fn calculate_end_bonus(&self) -> i32 {
        let mut count = 0;
        for i in 0..BOARD_SIZE {
            if self.board[i] != -1 {
                count += 1;
            }
        }

        if count >= 10 {
            0
        } else {
            let bonus = 2000 - (count as i32 * count as i32 * 20);
            if bonus > 0 { bonus } else { 0 }
        }
    }

    /// 检查是否还有可行动作
    pub fn has_moves(&self) -> bool {
        // 横向及其邻居
        for r in 0..HEIGHT {
            for c in 0..WIDTH {
                let idx = self.idx(r, c);
                let color = self.board[idx];
                if color == -1 {
                    continue;
                }

                // 检查右侧
                if c + 1 < WIDTH && self.board[self.idx(r, c + 1)] == color {
                    return true;
                }
                // 检查下方
                if r + 1 < HEIGHT && self.board[self.idx(r + 1, c)] == color {
                    return true;
                }
            }
        }
        false
    }
}
