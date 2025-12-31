import tkinter as tk
from tkinter import ttk, messagebox
import threading
import mss
import numpy as np
from PIL import Image, ImageTk
import os
import time
import subprocess
import shutil

from ai_model.predict import PopStarPredictor
from game.engine import PopStarEngine
from game.solver import PopStarSolver

class PopStarApp:
    def __init__(self, root):
        self.root = root
        self.root.title("消灭星星助手")
        # 移除固定大小，使用自适应
        
        # 1. 加载模型
        try:
            self.predictor = PopStarPredictor('weights/popstar_best.pth')
        except Exception as e:
            messagebox.showerror("错误", f"模型加载失败: {e}\n请先运行训练脚本。")
            self.root.destroy()
            return

        # 2. 素材
        self.assets = self._load_assets()
        
        # 3. 状态
        self.engine = PopStarEngine(board=np.full((10, 10), -1))
        self.roi = None
        self.last_full_img = None # 缓存上一次截取的全图
        self.planned_path = [] # 存储 AI 规划好的动作序列
        self.best_move = None
        self.is_analyzing = False

        self.setup_ui()

    def _load_assets(self):
        asset_map = {}
        names = ['蓝', '绿', '红', '紫', '黄']
        for i, name in enumerate(names):
            path = f"png/{name}.png"
            if os.path.exists(path):
                img = Image.open(path).convert("RGBA").resize((50, 50))
                asset_map[i] = ImageTk.PhotoImage(img)
        return asset_map

    def setup_ui(self):
        # 设置样式
        style = ttk.Style()
        style.configure("Big.TButton", font=("Arial", 12, "bold"), padding=10)
        
        # 1. 顶部基础控制栏
        top_frame = ttk.Frame(self.root, padding="10")
        top_frame.pack(fill=tk.X)
        
        ttk.Button(top_frame, text="1. 开始框选", command=self.select_roi).pack(side=tk.LEFT, padx=5)
        ttk.Button(top_frame, text="重新同步/识别", command=self.sync_board).pack(side=tk.LEFT, padx=5)
        ttk.Button(top_frame, text="重新计算路径", command=self.recalculate).pack(side=tk.LEFT, padx=5)
        
        self.status_label = ttk.Label(top_frame, text="就绪", foreground="blue")
        self.status_label.pack(side=tk.RIGHT, padx=5)

        # 2. 中间主体区：棋盘 + 侧边栏按钮
        main_content = ttk.Frame(self.root, padding="10")
        main_content.pack(fill=tk.BOTH, expand=True)

        # 左侧棋盘
        self.board_frame = ttk.LabelFrame(main_content, text="游戏引擎仿真")
        self.board_frame.pack(side=tk.LEFT, padx=5, fill=tk.BOTH)
        
        self.canvas = tk.Canvas(self.board_frame, bg="#1a1a1a", width=500, height=500, highlightthickness=0)
        self.canvas.pack(padx=5, pady=5)

        # 右侧操作区
        side_panel = ttk.Frame(main_content, padding="5")
        side_panel.pack(side=tk.LEFT, fill=tk.Y, padx=10)

        # 超大的下一步按钮
        self.btn_next = ttk.Button(side_panel, text="点我继续\n(下一步)", 
                                   command=self.execute_next_planned,
                                   style="Big.TButton")
        self.btn_next.pack(fill=tk.X, pady=(20, 10))
        
        ttk.Separator(side_panel, orient='horizontal').pack(fill=tk.X, pady=10)

        # 分数和信息展示
        info_frame = ttk.Frame(side_panel)
        info_frame.pack(fill=tk.X)
        
        self.score_label = ttk.Label(info_frame, text="积分: 0", font=("Arial", 14))
        self.score_label.pack(anchor=tk.W, pady=5)
        
        self.predicted_label = ttk.Label(info_frame, text="预计最大:\n --- ", 
                                         font=("Arial", 11, "bold"), foreground="#2e7d32")
        self.predicted_label.pack(anchor=tk.W, pady=5)

        self.render_board()

    def select_roi(self):
        img = self._get_screenshot()
        if not img: return
        
        self.last_full_img = img 
        
        roi_win = tk.Toplevel()
        roi_win.attributes("-fullscreen", True, "-topmost", True)
        canvas = tk.Canvas(roi_win, highlightthickness=0)
        canvas.pack(fill=tk.BOTH, expand=True)
        self.bg_img = ImageTk.PhotoImage(img)
        canvas.create_image(0, 0, image=self.bg_img, anchor=tk.NW)
        canvas.create_rectangle(0, 0, img.width, img.height, fill="black", stipple="gray25")
        start_x, start_y, rect_id = None, None, None
        def on_down(e): nonlocal start_x, start_y, rect_id; start_x, start_y = e.x, e.y; rect_id = canvas.create_rectangle(e.x, e.y, e.x, e.y, outline="cyan", width=2)
        def on_move(e): canvas.coords(rect_id, start_x, start_y, e.x, e.y)
        def on_up(e): 
            self.roi = (min(start_x, e.x), min(start_y, e.y), abs(e.x - start_x), abs(e.y - start_y))
            roi_win.destroy()
            self.root.deiconify()
            self.status_label.config(text="区域已锁定")
            # 自动触发第一次同步，且使用刚才 selection 时的缓存图
            self.sync_board(use_cache=True)
        canvas.bind("<ButtonPress-1>", on_down); canvas.bind("<B1-Motion>", on_move); canvas.bind("<ButtonRelease-1>", on_up)


    def _get_screenshot(self):
        # 1. 优先使用 MSS (高效、跨平台)
        try:
            with mss.mss() as sct:
                mon = sct.monitors[1] if len(sct.monitors) > 1 else sct.monitors[0]
                sct_img = sct.grab(mon)
                return Image.frombytes("RGB", sct_img.size, sct_img.bgra, "raw", "BGRX")
        except:
            pass

        # 2. 降级尝试 PyAutoGUI (如果已安装)
        try:
            import pyautogui
            return pyautogui.screenshot()
        except:
            pass
            
        # 3. 尝试 PIL ImageGrab
        try:
            from PIL import ImageGrab
            return ImageGrab.grab()
        except:
            pass
            
        # 4. 尝试 pyscreenshot (通用封装库，支持部分 Wayland)
        try:
            import pyscreenshot as ImageGrab
            return ImageGrab.grab()
        except:
            pass

        # 5. Wayland/系统命令回退 (关键修复)
        # 现代 Linux (Wayland) 禁止纯 Python 库截屏，必须调用系统工具
        tools = [
            (["spectacle", "-b", "-n", "-o", "/tmp/popstar_s.png"], "/tmp/popstar_s.png"),
            (["gnome-screenshot", "-f", "/tmp/popstar_s.png"], "/tmp/popstar_s.png"),
            (["scrot", "/tmp/popstar_s.png"], "/tmp/popstar_s.png"),
            (["grim", "/tmp/popstar_s.png"], "/tmp/popstar_s.png")
        ]
        
        for cmd, path in tools:
            if shutil.which(cmd[0]):
                try:
                    subprocess.run(cmd, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
                    if os.path.exists(path):
                        return Image.open(path).convert("RGB")
                except:
                    continue

        messagebox.showerror("错误", "无法获取屏幕截图。\n检测到可能在 Wayland 环境下，且未找到支持的截图工具(spectacle/gnome-screenshot/scrot/grim)。")
        return None

    def _get_roi_image(self, use_cache=False):
        if not self.roi: return None
        x, y, w, h = self.roi
        
        # 如果要求使用缓存（比如刚框选完），直接从缓存图裁剪
        if use_cache and self.last_full_img:
            return self.last_full_img.crop((x, y, x + w, y + h))

        try:
            with mss.mss() as sct:
                monitor = {"top": y, "left": x, "width": w, "height": h}
                # 抓取前更新一下缓存
                sct_img = sct.grab(monitor)
                # 注意: self.last_full_img 是完整截图，不仅仅是 ROI。
                # 如果我们使用 mss 只截取 ROI，就没有完整截图来更新 last_full_img。
                # 所以，last_full_img 仅在调用 _get_screenshot() 时更新。
                return Image.frombytes("RGB", sct_img.size, sct_img.bgra, "raw", "BGRX")
        except: pass
        
        full = self._get_screenshot()
        self.last_full_img = full
        return full.crop((x, y, x + w, y + h)) if full else None

    def sync_board(self, use_cache=False):
        if not self.roi: return
        self.status_label.config(text="正在全盘识别...")
        threading.Thread(target=self._run_sync, args=(use_cache,), daemon=True).start()

    def _run_sync(self, use_cache):
        roi_img = self._get_roi_image(use_cache=use_cache)
        if not roi_img: return
        w, h = roi_img.size
        cw, ch = w / 10, h / 10
        matrix = np.full((10, 10), -1, dtype=int)
        for r in range(10):
            for c in range(10):
                cell = roi_img.crop((c * cw, r * ch, (c + 1) * cw, (r + 1) * ch))
                v = self.predictor.predict_cell(cell)
                matrix[r, c] = v if v < 5 else -1
        self.engine = PopStarEngine(board=matrix)
        self.planned_path = []
        self.root.after(0, self.recalculate)

    def recalculate(self):
        if not self.engine.has_moves(): return
        self.status_label.config(text="AI 正在规划全局路径...")
        self.is_analyzing = True
        threading.Thread(target=self._run_solver, daemon=True).start()

    def _run_solver(self):
        solver = PopStarSolver(self.engine)
        move, score, path = solver.solve(iterations=2000) # 提高迭代次数以获得稳定长路径
        self.root.after(0, lambda: self._on_solver_done(move, score, path))

    def _on_solver_done(self, move, score, path):
        self.best_move = move
        self.planned_path = path
        self.is_analyzing = False
        self.status_label.config(text="全局路径已锁定")
        self.predicted_label.config(text=f"预计最大总分: {score}")
        self.render_board()

    def execute_next_planned(self):
        """执行规划中的下一步，点击后瞬发，不需要重新计算"""
        if self.is_analyzing: return
        if not self.planned_path:
            self.recalculate()
            return
        
        # 弹出第一步
        move = self.planned_path.pop(0)
        r, c = move
        
        # 如果当前位置非法（比如由于同步误差），则重新计算
        if self.engine.board[r, c] == -1 or len(self.engine.get_connected_group(r, c)) < 2:
            print("Detected path deviation, recalculating...")
            self.recalculate()
            return

        self.engine.eliminate(r, c)
        self.score_label.config(text=f"积分: {self.engine.total_score}")
        
        # 更新预览图中下一个推荐位置
        self.best_move = self.planned_path[0] if self.planned_path else None
        self.render_board()

    def render_board(self):
        self.canvas.delete("all")
        cs = 50
        for r in range(10):
            for c in range(10):
                v = self.engine.board[r, c]
                x0, y0 = c * cs, r * cs
                self.canvas.create_rectangle(x0, y0, x0+cs, y0+cs, outline="#333", fill="#222")
                if v != -1 and v in self.assets:
                    self.canvas.create_image(x0+cs//2, y0+cs//2, image=self.assets[v])
        if self.best_move:
            r, c = self.best_move
            x0, y0 = c * cs, r * cs
            self.canvas.create_rectangle(x0+2, y0+2, x0+cs-2, y0+cs-2, outline="red", width=4)

if __name__ == "__main__":
    root = tk.Tk()
    app = PopStarApp(root)
    root.mainloop()
