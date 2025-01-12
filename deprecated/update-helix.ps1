# Define Helix repository URL
$helixRepoUrl = "https://github.com/helix-editor/helix.git"

# Search for Helix directory from root with depth limit of 5
Write-Host "Searching for Helix..."
$helixPath = (fd -t d "helix" -d 5 -p "/" | Select-Object -First 1)
if (-not $helixPath) {
    Write-Host "Helix not found. Cloning repository..."
    git clone $helixRepoUrl "helix"
    $helixPath = Join-Path -Path (Get-Location) -ChildPath "helix"
}

# Build Helix from source
Write-Host "Building Helix from source..."
Push-Location $helixPath
cargo build --release
