import torch
import torch.nn as nn
import torch.nn.functional as F

class PopStarCNN(nn.Module):
    def __init__(self, num_classes=6):
        super(PopStarCNN, self).__init__()
        # 输入 64x64x3
        self.conv1 = nn.Conv2d(3, 16, kernel_size=3, padding=1)
        self.conv2 = nn.Conv2d(16, 32, kernel_size=3, padding=1)
        self.pool = nn.MaxPool2d(2, 2)
        
        self.fc1 = nn.Linear(32 * 16 * 16, 128)
        self.fc2 = nn.Linear(128, num_classes)
        self.dropout = nn.Dropout(0.25)

    def forward(self, x):
        # 64x64 -> 32x32
        x = self.pool(F.relu(self.conv1(x)))
        # 32x32 -> 16x16
        x = self.pool(F.relu(self.conv2(x)))
        
        x = x.view(-1, 32 * 16 * 16)
        x = self.dropout(F.relu(self.fc1(x)))
        x = self.fc2(x)
        return x

if __name__ == "__main__":
    model = PopStarCNN()
    dummy_input = torch.randn(1, 3, 64, 64)
    output = model(dummy_input)
    print(f"Output shape: {output.shape}")
