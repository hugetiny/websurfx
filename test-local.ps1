#!/usr/bin/env pwsh
# WebSurfX 本地测试脚本 (无需 Docker)
# 使用方法: .\test-local.ps1

$ErrorActionPreference = "Continue"
$websurfxDir = $PSScriptRoot

Write-Host "============================================" -ForegroundColor Cyan
Write-Host "  WebSurfX 本地测试" -ForegroundColor Cyan
Write-Host "============================================" -ForegroundColor Cyan
Write-Host ""

# 切换到 websurfx 目录
Set-Location $websurfxDir

# 1. 检查 Rust 工具链
Write-Host "[1/6] 检查 Rust 工具链..." -ForegroundColor Yellow
$rustVersion = rustc --version 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "错误: Rust 未安装，请先安装 Rust" -ForegroundColor Red
    exit 1
}
Write-Host "  Rust: $rustVersion" -ForegroundColor Green

# 2. 代码格式检查
Write-Host ""
Write-Host "[2/6] 检查代码格式..." -ForegroundColor Yellow
cargo fmt --all -- --check 2>&1 | Out-Null
if ($LASTEXITCODE -ne 0) {
    Write-Host "  警告: 代码格式需要调整，运行 'cargo fmt' 修复" -ForegroundColor Yellow
} else {
    Write-Host "  格式检查通过" -ForegroundColor Green
}

# 3. Clippy 检查 (使用 -p websurfx 指定包)
Write-Host ""
Write-Host "[3/6] 运行 Clippy..." -ForegroundColor Yellow
cargo clippy -p websurfx --all-targets 2>&1 | Select-Object -Last 5
if ($LASTEXITCODE -ne 0) {
    Write-Host "  警告: Clippy 有一些建议" -ForegroundColor Yellow
} else {
    Write-Host "  Clippy 检查通过" -ForegroundColor Green
}

# 4. 构建
Write-Host ""
Write-Host "[4/6] 构建项目..." -ForegroundColor Yellow
cargo build -p websurfx 2>&1 | Select-Object -Last 3
if ($LASTEXITCODE -ne 0) {
    Write-Host "  错误: 构建失败" -ForegroundColor Red
    exit 1
}
Write-Host "  构建成功" -ForegroundColor Green

# 5. 运行测试
Write-Host ""
Write-Host "[5/6] 运行单元测试..." -ForegroundColor Yellow
cargo test -p websurfx 2>&1 | Select-Object -Last 5
if ($LASTEXITCODE -ne 0) {
    Write-Host "  警告: 部分测试失败" -ForegroundColor Yellow
} else {
    Write-Host "  测试通过" -ForegroundColor Green
}

# 6. 构建 Release
Write-Host ""
Write-Host "[6/6] 构建 Release 版本..." -ForegroundColor Yellow
cargo build -p websurfx --release 2>&1 | Select-Object -Last 3
if ($LASTEXITCODE -ne 0) {
    Write-Host "  错误: Release 构建失败" -ForegroundColor Red
    exit 1
}
Write-Host "  Release 构建成功" -ForegroundColor Green

# 显示二进制信息
Write-Host ""
Write-Host "============================================" -ForegroundColor Cyan
Write-Host "  构建完成!" -ForegroundColor Cyan
Write-Host "============================================" -ForegroundColor Cyan

# 查找二进制文件
$binaryPath = Join-Path (Split-Path $websurfxDir -Parent) "target\release\websurfx.exe"
if (Test-Path $binaryPath) {
    $size = (Get-Item $binaryPath).Length / 1MB
    Write-Host "  二进制文件: $binaryPath"
    Write-Host "  文件大小: $([math]::Round($size, 2)) MB"
} else {
    Write-Host "  二进制文件位于: target\release\websurfx.exe"
}

Write-Host ""
Write-Host "所有测试完成!" -ForegroundColor Green
