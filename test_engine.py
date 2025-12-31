from game.engine import PopStarEngine
import numpy as np

def test_engine():
    # 创建一个特定的测试场景
    # 0 0 .
    # 1 0 .
    # 0 1 .
    test_board = np.full((10, 10), -1)
    test_board[9, 0] = 0
    test_board[8, 0] = 1
    test_board[7, 0] = 0
    test_board[9, 1] = 1
    test_board[8, 1] = 0
    test_board[7, 1] = 0
    
    engine = PopStarEngine(board=test_board)
    print("初始状态:")
    print(engine)
    
    print("\n在 (7, 1) 处执行消除 - 颜色 0")
    score = engine.eliminate(7, 1)
    print(f"消除得分: {score}")
    print("消除及重力下落后的状态:")
    print(engine)
    
    remaining = engine.get_remaining_count()
    print(f"剩余星星数: {remaining}")
    
    if engine.has_moves():
        print("仍有可消除动作。")
    else:
        print("无动可走。结束奖励:", engine.calculate_end_bonus())

if __name__ == "__main__":
    test_engine()
