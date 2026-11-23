param(
    [string]$Version
)

$ErrorActionPreference = "Stop"

if (-not $Version) {
    # Read current version from Cargo.toml
    $cargoContent = Get-Content .\Cargo.toml -Raw
    if ($cargoContent -match '(?m)^version\s*=\s*"([^"]+)"') {
        $Version = $matches[1]
    } else {
        Write-Error "Could not determine version from Cargo.toml. Please specify version, e.g.: .\release.ps1 -Version 2.22.0"
        exit 1
    }
}

# Ensure tag has leading 'v'
$tag = if ($Version.StartsWith("v")) { $Version } else { "v$Version" }

Write-Host "Creating and pushing release tag: $tag ..." -ForegroundColor Cyan

# Check if tag already exists locally
$existingLocal = git tag -l $tag
if ($existingLocal) {
    Write-Warning "Tag $tag already exists locally. Overwriting/re-tagging..."
    git tag -d $tag
}

# Create tag and push
git tag -a $tag -m "Release $tag"
git push origin $tag

Write-Host "Successfully pushed $tag to GitHub!" -ForegroundColor Green
Write-Host "GitHub Actions is now building and publishing the release assets." -ForegroundColor Green
