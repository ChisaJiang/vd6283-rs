# STM32 Cargo Size 美化显示

# 修改成你的 STM32 实际 Flash / RAM 大小
$FLASH_KB = 512
$RAM_KB   = 128

# 获取 cargo size
$output = cargo size --release | Select-Object -Last 1

# 解析：
# text data bss dec hex filename
if ($output -match '^\s*(\d+)\s+(\d+)\s+(\d+)\s+(\d+)\s+([0-9a-fA-F]+)\s+(.+)$') {

    $text = [int]$matches[1]
    $data = [int]$matches[2]
    $bss  = [int]$matches[3]

    # Flash = text + data
    $flash = $text + $data

    # RAM = data + bss
    $ram = $data + $bss

    # 转 KiB
    $textKB  = $text / 1024
    $dataKB  = $data / 1024
    $bssKB   = $bss / 1024
    $flashKB = $flash / 1024
    $ramKB   = $ram / 1024

    # 使用率
    $flashUsage = ($flash / ($FLASH_KB * 1024)) * 100
    $ramUsage   = ($ram / ($RAM_KB * 1024)) * 100

    Write-Host ""
    Write-Host "========== STM32 Memory Usage =========="

    Write-Host ("Flash              {0,8:N2} KiB / {1} KiB" -f $flashKB, $FLASH_KB)
    Write-Host ("Usage              {0,8:N2} %" -f $flashUsage)

    Write-Host "  --------------------------------------"
    Write-Host ("RAM                {0,8:N2} KiB / {1} KiB" -f $ramKB, $RAM_KB)
    Write-Host ("Usage              {0,8:N2} %" -f $ramUsage)

    Write-Host "========================================="
    Write-Host ""
}
