#!/bin/bash

# 检查是否存在 ci-user 目录，如果存在则删除
if [ -d "ci-user" ]; then
    echo "Removing existing ci-user directory..."
    rm -rf ci-user
fi

# 克隆 rCore-Tutorial-Checker-2025S 到 ci-user 目录
echo "Cloning rCore-Tutorial-Checker-2025S..."
git clone git@github.com:LearningOS/rCore-Tutorial-Checker-2025S ci-user

# 克隆 rCore-Tutorial-Test-2025S 到 ci-user/user 目录
echo "Cloning rCore-Tutorial-Test-2025S into ci-user/user..."
git clone git@github.com:LearningOS/rCore-Tutorial-Test-2025S ci-user/user

# 进入 ci-user 目录
cd ci-user

echo "All operations completed."

