
import time
from game.engine import PopStarEngine
from game.solver import PopStarSolver

def run_benchmark(iterations=50000, silent=False):
    """
    运行性能测试并返回 IPS (每秒迭代次数)
    """
    engine = PopStarEngine()
    solver = PopStarSolver(engine)
    
    if not silent:
        print(f"正在运行性能测试，模拟次数: {iterations} ...")
    
    start_time = time.time()
    # 执行求解
    move, score, path = solver.solve(iterations=iterations)
    end_time = time.time()
    
    duration = end_time - start_time
    ips = iterations / duration
    
    if not silent:
        print(f"模拟次数: {iterations}")
        print(f"耗时: {duration:.4f}秒")
        print(f"性能 (IPS): {ips:.2f} 次/秒")
        print(f"推荐移动: {move}, 预测总分: {score}")
        
    return ips

if __name__ == "__main__":
    run_benchmark()
