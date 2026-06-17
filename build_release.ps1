Write-Host "Building far1c for FAR 3 (Windows)..."
cargo build --release

Write-Host "Packaging release..."
$TargetDir = "target\release\far3"
if (!(Test-Path $TargetDir)) {
    New-Item -ItemType Directory -Force -Path $TargetDir | Out-Null
}

Copy-Item "target\release\far1c.dll" -Destination "$TargetDir\far1c.dll" -Force
Copy-Item "dist\*.lng" -Destination "$TargetDir\" -Force

$FarDir = "C:\Program Files\Far Manager"
$Far1cPluginDir = "$FarDir\Plugins\Far1C"

if (Test-Path $FarDir) {
    if (!(Test-Path $Far1cPluginDir)) {
        Write-Host "Creating junction point in Far Manager plugins directory..."
        $AbsoluteTarget = (Get-Item $TargetDir).FullName
        try {
            New-Item -ItemType Junction -Path $Far1cPluginDir -Target $AbsoluteTarget -ErrorAction Stop | Out-Null
            Write-Host "Junction point created successfully."
        }
        catch {
            Write-Host "Failed to create junction point. Please run the script as Administrator." -ForegroundColor Yellow
        }
    }
}

Write-Host "Build complete in $TargetDir"
