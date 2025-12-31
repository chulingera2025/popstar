import torch
import cv2
import numpy as np
from PIL import Image
from torchvision import transforms
from ai_model.model import PopStarCNN

class PopStarPredictor:
    def __init__(self, weight_path='weights/popstar_best.pth'):
        self.device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
        self.model = PopStarCNN(num_classes=6).to(self.device)
        self.model.load_state_dict(torch.load(weight_path, map_location=self.device))
        self.model.eval()
        
        self.transform = transforms.Compose([
            transforms.Resize((64, 64)),
            transforms.ToTensor(),
            transforms.Normalize((0.5, 0.5, 0.5), (0.5, 0.5, 0.5))
        ])
        self.classes = ['蓝', '绿', '红', '紫', '黄', '空']

    def predict_cell(self, cell_img):
        """预测单个格子的颜色"""
        # cell_img 为 PIL Image
        if cell_img.mode != 'RGB':
            cell_img = cell_img.convert('RGB')
        img_tensor = self.transform(cell_img).unsqueeze(0).to(self.device)
        with torch.no_grad():
            output = self.model(img_tensor)
            _, predicted = torch.max(output, 1)
        return predicted.item()

    def predict_board(self, screenshot_path, grid_box):
        """
        从截图和格子区域识别整个棋盘
        grid_box: (x, y, w, h)
        """
        img = Image.open(screenshot_path).convert('RGB')
        x, y, w, h = grid_box
        grid_img = img.crop((x, y, x + w, y + h))
        
        cell_w = w / 10
        cell_h = h / 10
        
        matrix = np.full((10, 10), -1, dtype=int)
        
        for r in range(10):
            for c in range(10):
                left = c * cell_w
                top = r * cell_h
                right = left + cell_w
                bottom = top + cell_h
                
                cell_img = grid_img.crop((left, top, right, bottom))
                class_idx = self.predict_cell(cell_img)
                
                # 0-4 对应颜色, 5 对应空
                if class_idx < 5:
                    matrix[r, c] = class_idx
                else:
                    matrix[r, c] = -1
        return matrix

if __name__ == "__main__":
    # 示例用法
    # predictor = PopStarPredictor()
    # board = predictor.predict_board('screenshot.png', (100, 200, 500, 500))
    # print(board)
    pass
