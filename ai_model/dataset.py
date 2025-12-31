import os
import cv2
import numpy as np
import torch
from torch.utils.data import Dataset, DataLoader
from PIL import Image
import glob

class PopStarDataset(Dataset):
    """
    用于训练的消灭星星数据集
    从 png/ 文件夹加载 5 种基础图片，并进行增强产生数据
    """
    def __init__(self, root_dir='png', transform=None, num_samples_per_color=200):
        self.root_dir = root_dir
        self.transform = transform
        self.classes = ['蓝', '绿', '红', '紫', '黄', '空'] # 5色 + 1空
        self.data = []
        self.labels = []
        
        # 加载基础图像
        base_images = {}
        paths = glob.glob(os.path.join(root_dir, '*.png'))
        for p in paths:
            name = os.path.basename(p).split('.')[0]
            # 读取为 RGB
            img = Image.open(p).convert('RGB')
            base_images[name] = img
            
        # 生成数据
        for i, cls_name in enumerate(self.classes):
            if cls_name == '空':
                # 生成黑色或杂色背景作为空位
                for _ in range(num_samples_per_color):
                    # 随机背景
                    bg = np.zeros((128, 128, 3), dtype=np.uint8)
                    if np.random.rand() > 0.5:
                        bg += np.random.randint(0, 30, (128, 128, 3), dtype=np.uint8)
                    self.data.append(Image.fromarray(bg))
                    self.labels.append(i)
                continue
            
            if cls_name not in base_images:
                continue
                
            base_img = base_images[cls_name]
            for _ in range(num_samples_per_color):
                # 对基础图进行随机增强 (旋转, 缩放, 亮度等会在 transform 中处理)
                # 这里我们直接存基础图，靠 transform 实现多样性
                self.data.append(base_img)
                self.labels.append(i)

    def __len__(self):
        return len(self.data)

    def __getitem__(self, idx):
        img = self.data[idx]
        label = self.labels[idx]
        
        if self.transform:
            img = self.transform(img)
            
        return img, label

if __name__ == "__main__":
    # 测试数据集加载
    from torchvision import transforms
    transform = transforms.Compose([
        transforms.Resize((64, 64)),
        transforms.RandomRotation(10),
        transforms.ColorJitter(brightness=0.2, contrast=0.2),
        transforms.ToTensor(),
    ])
    ds = PopStarDataset(root_dir='png', transform=transform)
    print(f"Dataset size: {len(ds)}")
    img, label = ds[0]
    print(f"Sample shape: {img.shape}, label: {label}")
