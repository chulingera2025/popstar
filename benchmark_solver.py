
import time
from game.engine import PopStarEngine
from game.solver import PopStarSolver

def benchmark():
    # 初始化引擎和求解器
    engine = PopStarEngine()
    solver = PopStarSolver(engine)
    
    start_time = time.time()
    iterations = 50000
    print(f"正在运行性能测试，模拟次数: {iterations} ...")
    
    # 执行求解
    # 注意：这里的 iterations 参数传递给 Rust 后端的 MCTS 模拟次数
    move, score, path = solver.solve(iterations=iterations)
    
    end_time = time.time()
    duration = end_time - start_time
    
    print(f"模拟次数: {iterations}")
    print(f"耗时: {duration:.4f}秒")
    print(f"性能 (IPS): {iterations/duration:.2f} 次/秒")
    print(f"推荐移动: {move}, 预测总分: {score}")

if __name__ == "__main__":
    benchmark()
