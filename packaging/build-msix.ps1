<#
.SYNOPSIS
    Stages, and where possible packs, the MSIX for the Microsoft Store.

.DESCRIPTION
    Produces a layout under dist\msix-layout and, if makeappx.exe from the
    Windows SDK is on hand, packs it into dist\CapsLang-<version>.msix.

    The package is left unsigned on purpose. The Store re-signs submissions
    with a Microsoft-trusted certificate, which is the whole reason to go this
    route: it is what clears SmartScreen and Smart App Control. To try the
    layout locally without signing anything, turn on Developer Mode and run:

        Add-AppxPackage -Register dist\msix-layout\AppxManifest.xml

.PARAMETER Version
    Four-part package version. The Store requires the last part to be 0.

.PARAMETER IdentityName
    Package identity from Partner Center. Keep the default until the name is
    reserved; a local test only needs it to be well-formed.

.PARAMETER Publisher
    Publisher distinguished name. Must match the signing certificate exactly,
    and for a Store submission that is the DN Partner Center shows you.
#>
[CmdletBinding()]
param(
    [string]$Version = "0.1.0.0",
    [string]$BinDir = "target\release",
    [string]$IdentityName = "technocoluzi.CapsLang",
    [string]$Publisher = "CN=technocoluzi",
    [string]$PublisherDisplayName = "technocoluzi"
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Push-Location $root
try {
    if ($Version -notmatch '^\d+\.\d+\.\d+\.\d+$') {
        throw "Version must have four parts, e.g. 0.1.0.0 (got '$Version')"
    }
    if (-not $Version.EndsWith(".0")) {
        Write-Warning "The Store rejects packages whose fourth version part is not 0."
    }

    $exe = Join-Path $root "$BinDir\capslang.exe"
    if (-not (Test-Path $exe)) { throw "Build first: $exe not found" }

    $layout = Join-Path $root "dist\msix-layout"
    if (Test-Path $layout) { Remove-Item $layout -Recurse -Force }
    New-Item -ItemType Directory -Force $layout | Out-Null

    Copy-Item $exe (Join-Path $layout "capslang.exe")
    Copy-Item (Join-Path $root "packaging\msix\Assets") $layout -Recurse

    (Get-Content (Join-Path $root "packaging\msix\AppxManifest.xml") -Raw).
        Replace("__IDENTITY_NAME__", $IdentityName).
        Replace("__PUBLISHER__", $Publisher).
        Replace("__PUBLISHER_DISPLAY_NAME__", $PublisherDisplayName).
        Replace("__VERSION__", $Version) |
        Set-Content (Join-Path $layout "AppxManifest.xml") -Encoding utf8

    Write-Host "Staged layout: $layout"

    $makeappx = Get-ChildItem "C:\Program Files (x86)\Windows Kits\10\bin" -Recurse -Filter makeappx.exe -ErrorAction SilentlyContinue |
        Sort-Object FullName -Descending | Select-Object -First 1
    if (-not $makeappx) {
        Write-Host ""
        Write-Host "makeappx.exe not found (install the Windows SDK to pack a .msix)."
        Write-Host "To test the layout locally, enable Developer Mode and run:"
        Write-Host "  Add-AppxPackage -Register `"$layout\AppxManifest.xml`""
        return
    }

    $out = Join-Path $root "dist\CapsLang-$Version.msix"
    if (Test-Path $out) { Remove-Item $out -Force }
    & $makeappx.FullName pack /d $layout /p $out /o
    if ($LASTEXITCODE -ne 0) { throw "makeappx failed with exit code $LASTEXITCODE" }
    Write-Host "Packed: $out"
}
finally {
    Pop-Location
}
