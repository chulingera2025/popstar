import numpy as np

class PopStarEngine:
    """
    消灭星星核心引擎
    矩阵定义: 10x10, -1 表示空位, 0-4 表示五种颜色
    """
    WIDTH = 10
    HEIGHT = 10

    def __init__(self, board=None):
        if board is not None:
            self.board = np.array(board, dtype=int)
        else:
            # 随机初始化棋盘 (0-4)
            self.board = np.random.randint(0, 5, (self.HEIGHT, self.WIDTH))
        self.score = 0
        self.total_score = 0

    def copy(self):
        new_engine = PopStarEngine(board=self.board.copy())
        new_engine.score = self.score
        new_engine.total_score = self.total_score
        return new_engine

    def get_connected_group(self, r, c):
        """获取 (r, c) 坐标所在的同色连通区域"""
        color = self.board[r, c]
        if color == -1:
            return set()
        
        group = set()
        stack = [(r, c)]
        group.add((r, c))

        while stack:
            curr_r, curr_c = stack.pop()
            for dr, dc in [(0, 1), (0, -1), (1, 0), (-1, 0)]:
                nr, nc = curr_r + dr, curr_c + dc
                if 0 <= nr < self.HEIGHT and 0 <= nc < self.WIDTH:
                    if self.board[nr, nc] == color and (nr, nc) not in group:
                        group.add((nr, nc))
                        stack.append((nr, nc))
        return group

    def eliminate(self, r, c, known_group=None):
        """执行消除逻辑，返回本次消除得分"""
        if self.board[r, c] == -1:
            return 0
        
        group = known_group if known_group is not None else self.get_connected_group(r, c)
        n = len(group)
        
        if n < 2:
            return 0
        
        # 1. 计算得分
        move_score = n * n * 5
        self.score += move_score
        self.total_score += move_score

        # 2. 标记消除
        for gr, gc in group:
            self.board[gr, gc] = -1
        
        # 3. 应用重力 (方块掉落)
        self._apply_gravity()
        
        # 4. 列合并 (向左合并空列)
        self._apply_column_shift()
        
        return move_score

    def _apply_gravity(self):
        """每一列方块向下掉落"""
        for c in range(self.WIDTH):
            # 获取当前列所有非空元素
            col = self.board[:, c]
            remaining = col[col != -1]
            # 填充到最底端
            new_col = np.full(self.HEIGHT, -1, dtype=int)
            new_col[self.HEIGHT - len(remaining):] = remaining
            self.board[:, c] = new_col

    def _apply_column_shift(self):
        """如果某一列全空，右侧列整体向左移"""
        non_empty_cols = []
        for c in range(self.WIDTH):
            if not np.all(self.board[:, c] == -1):
                non_empty_cols.append(self.board[:, c])
        
        # 重新构建棋盘
        new_board = np.full((self.HEIGHT, self.WIDTH), -1, dtype=int)
        for i, col_data in enumerate(non_empty_cols):
            new_board[:, i] = col_data
        self.board = new_board

    def get_remaining_count(self):
        """获取剩余星星数量"""
        return np.sum(self.board != -1)

    def calculate_end_bonus(self):
        """计算关卡结束奖励"""
        rem = self.get_remaining_count()
        if rem >= 10:
            return 0
        bonus = 2000 - (rem * rem * 20)
        return max(0, bonus)

    def has_moves(self):
        """判断是否还有可消除的动作"""
        for r in range(self.HEIGHT):
            for c in range(self.WIDTH):
                if self.board[r, c] != -1:
                    # 检查相邻
                    color = self.board[r, c]
                    for dr, dc in [(0, 1), (1, 0)]:
                        nr, nc = r + dr, c + dc
                        if 0 <= nr < self.HEIGHT and 0 <= nc < self.WIDTH:
                            if self.board[nr, nc] == color:
                                return True
        return False

    def __str__(self):
        res = []
        for r in range(self.HEIGHT):
            row_str = " ".join([str(x) if x != -1 else "." for x in self.board[r]])
            res.append(row_str)
        return "\n".join(res)
