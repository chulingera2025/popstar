# PopStar 助手

这是一个针对"消灭星星" (PopStar) 游戏的智能辅助工具。它结合了计算机视觉 (CV) 和蒙特卡洛树搜索 (MCTS) 算法，能够自动识别屏幕上的游戏状态，并规划出最优消除路径以获得最高分。

为了追求极致的计算性能，核心算法引擎（消除逻辑、MCTS 求解器）已完全使用 **Rust 2024** 重写，并通过 PyO3 无缝集成到 Python 中。

## ✨ 主要特性

- **🚀 Rust 高性能引擎**: 核心算法采用 Rust 编写，计算速度比纯 Python 版本快 **1000 倍** (从 ~200 IPS 提升至 ~280,000+ IPS)，能够在毫秒级内进行深层搜索。
- **👁️ 自动化视觉识别**: 使用 PyTorch/CNN 训练的模型自动识别屏幕方块颜色，无需人工干预。
- **🤖 智能路径规划**: 基于 MCTS (蒙特卡洛树搜索) 算法，不仅看当前步，还能预判后续掉落和合并，追求全局最高分。
- **🖥️ 交互式 GUI**: 提供直观的图形界面，支持区域框选、实时预览、手动/自动执行下一步。
- **🔌 无缝集成**: Python 前端与 Rust 后端自动桥接，无需复杂配置，体验如丝般顺滑。

## 🛠️ 项目结构

```
.
├── ai_model/           # PyTorch 模型定与训练脚本 (CV部分)
│   ├── model.py        # CNN 网络结构
│   ├── train.py        # 训练脚本
│   └── predict.py      # 预测/推理接口
├── game/               # 游戏逻辑封装
│   ├── solver.py       # 求解器入口 (调用 Rust 后端)
│   └── engine.py       # Python 版引擎 (已废弃/仅作参考)
├── popstar_rs/         # Rust 核心扩展库源码
│   ├── src/
│   │   ├── engine.rs   # 游戏其引擎 (Result 2024)
│   │   ├── solver.rs   # MCTS 求解器 (Rust 2024)
│   │   └── lib.rs      # PyO3 绑定入口
│   └── Cargo.toml      # Rust 项目配置
├── main.py             # GUI 主程序入口
├── benchmark_solver.py # 性能测试脚本
└── requirements.txt    # Python 依赖列表
```

## ⚙️ 安装与配置

### 前置要求
- Python 3.10+
- Rust 工具链 (建议最新 stable)

### 1. 安装 Python 依赖
```bash
pip install -r requirements.txt
```

### 2. 编译并安装 Rust 扩展
为了获得最佳性能，需要编译 Rust 后端：
```bash
cd popstar_rs
maturin develop --release
cd ..
```
*注: `maturin develop` 会将 Rust 扩展直接编译安装到当前的 Python 环境中。*

## 🚀 使用指南

1. **启动程序**
   ```bash
   python main.py
   ```

2. **操作流程**
   - 点击 **"1. 开始框选"**：在屏幕上框选消灭星星的游戏区域。
   - 程序会自动识别并同步棋盘状态到左侧"游戏引擎仿真"区。
   - 此时 AI 已在后台疯狂计算最佳路径（利用 Rust 引擎）。
   - 点击 **"点我继续 (下一步)"**：执行 AI 推荐的一步操作。
   - 或者观察右侧信息栏的 **"预计最大总分"**。

## 🧪 性能测试

你可以运行 `benchmark_solver.py` 来测试当前环境下的求解速度：
```bash
python benchmark_solver.py
```
典型结果 (MCTS 50,000 次模拟):
- **Rust 版本**: ~0.17 秒 (~280,000 IPS)
- **Python 版本**: ~180 秒 (~270 IPS)

## 📝 开发说明

- **Rust 代码**: 位于 `popstar_rs/`，修改后需重新运行 `maturin develop`。
- **注释**: 关键代码均包含详细中文注释

## 📄 License

MIT
