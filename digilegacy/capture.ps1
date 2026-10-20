$ErrorActionPreference = 'Stop'
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class Win {
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int cmd);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr after, int x, int y, int cx, int cy, uint flags);
  [DllImport("user32.dll")] public static extern void SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint flag, uint dx, uint dy, uint data, IntPtr extra);
  [DllImport("user32.dll")] public static extern int GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetWindowText(IntPtr h, System.Text.StringBuilder s, int n);
  [DllImport("user32.dll")] public static extern bool EnumWindows(EnumWindowsProc cb, IntPtr e);
  public delegate bool EnumWindowsProc(IntPtr h, IntPtr e);
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  public static void Click(int x, int y) {
    SetCursorPos(x, y);
    System.Threading.Thread.Sleep(80);
    mouse_event(0x02, 0,0,0, IntPtr.Zero);
    System.Threading.Thread.Sleep(60);
    mouse_event(0x04, 0,0,0, IntPtr.Zero);
  }
}
"@
[void][System.Reflection.Assembly]::LoadWithPartialName('System.Drawing')

# ── 导航坐标（CSS 像素，以 1200px 基准窗口宽度计） ──────────────
# 数值按当前 main.css 推导：原生标题栏约 32px + 侧边栏头部（24+40+24）+ 导航容器
# padding 12px → 首项中心约 156px；.nav-item 高约 46px（12px 上下 padding + 22px 行盒）。
# 注意：仓库中的旧截图来自更早的构建（导航间距 ~107px），不能作为当前 UI 的校准依据。
# 若点击落空，请先截一张 01-dashboard，用像素测出首项中心 y 与相邻项间距，回填下面两个常量。
$NAV_FIRST_Y  = 156
$NAV_SPACING  = 46
$NAV_CLICK_X  = 130

# 侧边栏导航顺序（与 App.vue navRoutes 一致）：
#   0=仪表板 1=资产发现 2=我的资产 3=我的档案 4=安心传承 5=继承报告 6=继承人向导 7=法律模板
$wantTitle = "归零 - 数字遗产管理系统"
$pids = (Get-Process digilegacy -ErrorAction SilentlyContinue).Id | ForEach-Object { $_.ToString() }
if ($pids.Count -eq 0) { throw "digilegacy process not running" }

$found = New-Object System.Collections.ArrayList
$cb = [Win+EnumWindowsProc]{
  param($h, $e)
  $pid2 = 0
  [Win]::GetWindowThreadProcessId($h, [ref]$pid2) | Out-Null
  if ($pids -contains $pid2.ToString() -and [Win]::IsWindowVisible($h)) {
    $sb = New-Object System.Text.StringBuilder(512)
    [void][Win]::GetWindowText($h, $sb, 512)
    if ($sb.ToString() -eq $wantTitle) { $null = $found.Add($h) }
  }
  return $true
}
[void][Win]::EnumWindows($cb, [IntPtr]::Zero)
if ($found.Count -eq 0) { throw "window not found" }
$h = $found[0]

[Win]::ShowWindow($h, 9) | Out-Null   # SW_RESTORE（窗口最小化时先还原）
[Win]::SetWindowPos($h, [IntPtr]::Zero, 80, 80, 0, 0, 0x0001 -bor 0x0002) | Out-Null  # SWP_NOSIZE|SWP_NOZORDER
[Win]::SetForegroundWindow($h) | Out-Null
Start-Sleep -Milliseconds 400
# 最小化→还原 强制 WebView 重绘，避免首帧黑屏
[Win]::ShowWindow($h, 6) | Out-Null   # SW_MINIMIZE
Start-Sleep -Milliseconds 500
[Win]::ShowWindow($h, 9) | Out-Null   # SW_RESTORE
[Win]::SetForegroundWindow($h) | Out-Null
Start-Sleep -Milliseconds 1500

# 输出目录：脚本所在目录的 ../docs/screenshots（官网唯一截图来源）
$outDir = Join-Path (Split-Path $PSScriptRoot -Parent) "docs\screenshots"
if (-not (Test-Path $outDir)) { New-Item -ItemType Directory $outDir | Out-Null }

function GetRect {
  $r = New-Object Win+RECT
  [Win]::GetWindowRect($h, [ref]$r) | Out-Null
  $r
}

function Capture($name) {
  [Win]::SetForegroundWindow($h) | Out-Null
  Start-Sleep -Milliseconds 900
  $r = GetRect
  $w = $r.R - $r.L; $ht = $r.B - $r.T
  if ($w -lt 400 -or $ht -lt 300) { Write-Host "WARNING: window rect too small ($w x $ht), capture may be wrong"; return }
  $bmp = New-Object System.Drawing.Bitmap($w, $ht)
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.CopyFromScreen($r.L, $r.T, 0, 0, (New-Object System.Drawing.Size($w, $ht)))
  $g.Dispose()
  $path = Join-Path $outDir "$name.png"
  $bmp.Save($path, [System.Drawing.Imaging.ImageFormat]::Png)
  $bmp.Dispose()
  Write-Host "captured $path ($w x $ht)"
}

function Nav($index, $name) {
  $r = GetRect
  $scale = ($r.R - $r.L) / 1200.0
  $cx = $r.L + [int]($NAV_CLICK_X * $scale)
  $cy = $r.T + [int](($NAV_FIRST_Y + $index * $NAV_SPACING) * $scale)
  [Win]::Click($cx, $cy)
  Start-Sleep -Milliseconds 600
  Capture $name
}

Start-Sleep -Milliseconds 1500
Capture "01-dashboard"
Nav 1 "02-discover"
Nav 2 "03-assets"
Nav 3 "04-profile"
Nav 4 "05-dms"
Nav 5 "08-report"
Nav 6 "06-heir-guide"
Nav 7 "07-legal"
Write-Host "DONE"
