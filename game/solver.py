try:
    import popstar_rs
except ImportError:
    raise ImportError("Rust 扩展 module 'popstar_rs' 未找到。请先编译并安装：maturin develop --release")

class PopStarSolver:
    def __init__(self, engine):
        self.engine = engine

    def solve(self, iterations=1000):
        """
        使用 Rust 高性能求解器计算最佳移动。
        
        参数:
            iterations (int): 模拟次数 (在 Rust 端通过 MCTS 迭代)
            
        返回:
            (move, max_score, path): 最佳移动, 预测总分, 完整路径
        """
        # 转换板子数据: numpy (10,10) -> flat list
        board_data = self.engine.board.flatten().tolist()
        
        # 创建 Rust 引擎快照
        rs_engine = popstar_rs.PyPopStarEngine(board_data)
        
        # 修正 Rust 引擎的分数，确保返回的总分正确
        current_base_score = self.engine.total_score
        
        rs_solver = popstar_rs.PyPopStarSolver()
        move, rs_score, path = rs_solver.solve(rs_engine, iterations)
        
        return move, current_base_score + rs_score, path

if __name__ == "__main__":
    from game.engine import PopStarEngine
    engine = PopStarEngine()
    solver = PopStarSolver(engine)
    # Rust solver 极快，可以增加迭代次数
    move, score, path = solver.solve(iterations=10000)
    print(f"Move: {move}, Max Predicted: {score}, Path Length: {len(path)}")
