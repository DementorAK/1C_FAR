Write-Host "Building far1c for FAR 3 (Windows)..."
cargo build --release

Write-Host "Packaging release..."
$TargetDir = "target\release\far3"
if (!(Test-Path $TargetDir)) {
    New-Item -ItemType Directory -Force -Path $TargetDir | Out-Null
}

Copy-Item "target\release\far1c.dll" -Destination "$TargetDir\far1c.dll" -Force
Copy-Item "dist\*.lng" -Destination "$TargetDir\" -Force

Write-Host "Build complete in $TargetDir"
