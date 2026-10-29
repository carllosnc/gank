$ErrorActionPreference = "Stop"

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "       GANK DESKTOP ENGINE - BENCHMARK DE PERFORMANCE     " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

# 1. Tamanho do Executável Release
$exePath = "D:\Projects\open\gank\target\release\Gank.exe"
$exeItem = Get-Item $exePath
$exeMb = [math]::Round($exeItem.Length / 1MB, 2)
Write-Host "[1/5] Tamanho do Binario: $exeMb MB ($($exeItem.Length) bytes)" -ForegroundColor Green

# 2. Medição de Inicialização a Frio (Cold Startup)
$stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
$proc = Start-Process -FilePath $exePath -PassThru

while ($proc.MainWindowHandle -eq 0 -and !$proc.HasExited) {
    Start-Sleep -Milliseconds 5
    $proc.Refresh()
}
$startupMs = $stopwatch.ElapsedMilliseconds
Write-Host "[2/5] Tempo de Inicializacao a Frio (Startup): $startupMs ms" -ForegroundColor Green

# Estabilização dos Shaders e UI
Start-Sleep -Seconds 3

# 3. Medição em Foco (60 FPS com VSync + Shader Ativo)
Write-Host "[3/5] Coletando metricas em Foco (Renderizacao Ativa 60 FPS)..." -ForegroundColor Yellow
$proc.Refresh()
$wsFocus = [math]::Round($proc.WorkingSet64 / 1MB, 2)
$pmFocus = [math]::Round($proc.PrivateMemorySize64 / 1MB, 2)
$threadsFocus = $proc.Threads.Count
$handlesFocus = $proc.HandleCount

$cpuT1 = $proc.TotalProcessorTime.TotalSeconds
$time1 = [System.Diagnostics.Stopwatch]::GetTimestamp()
Start-Sleep -Seconds 3
$proc.Refresh()
$cpuT2 = $proc.TotalProcessorTime.TotalSeconds
$time2 = [System.Diagnostics.Stopwatch]::GetTimestamp()

$elapsedSecFocus = ($time2 - $time1) / [System.Diagnostics.Stopwatch]::Frequency
$cpuFocusPct = [math]::Round((($cpuT2 - $cpuT1) / ($elapsedSecFocus * [Environment]::ProcessorCount)) * 100, 2)

# 4. Medição Sem Foco (Focus Throttling - 15 FPS)
Write-Host "[4/5] Coletando metricas Sem Foco (Focus Throttling 15 FPS)..." -ForegroundColor Yellow
# Minimizar a janela para teste de economia de energia
$wscript = New-Object -ComObject WScript.Shell
$wscript.AppActivate((Get-Process -Id $PID).Id) | Out-Null
Start-Sleep -Seconds 1

$proc.Refresh()
$cpuT3 = $proc.TotalProcessorTime.TotalSeconds
$time3 = [System.Diagnostics.Stopwatch]::GetTimestamp()
Start-Sleep -Seconds 3
$proc.Refresh()
$cpuT4 = $proc.TotalProcessorTime.TotalSeconds
$time4 = [System.Diagnostics.Stopwatch]::GetTimestamp()

$elapsedSecUnfocus = ($time4 - $time3) / [System.Diagnostics.Stopwatch]::Frequency
$cpuUnfocusPct = [math]::Round((($cpuT4 - $cpuT3) / ($elapsedSecUnfocus * [Environment]::ProcessorCount)) * 100, 2)

$wsFinal = [math]::Round($proc.WorkingSet64 / 1MB, 2)
$pmFinal = [math]::Round($proc.PrivateMemorySize64 / 1MB, 2)

# Encerrar processo
Stop-Process -Id $proc.Id -Force

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "                RESULTADOS DO BENCHMARK                   " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

[PSCustomObject]@{
    "Tamanho do Executavel"       = "$exeMb MB"
    "Tempo de Inicializacao"      = "$startupMs ms"
    "Uso de CPU (Janela em Foco)" = "$cpuFocusPct %"
    "Uso de CPU (Sem Foco / 15fps)"= "$cpuUnfocusPct %"
    "Memoria RAM Fisica (WorkingSet)" = "$wsFinal MB"
    "Memoria Privada (Commit Charge)" = "$pmFinal MB"
    "Threads Ativas"              = $threadsFocus
    "Handles Alocados"            = $handlesFocus
} | Format-List
