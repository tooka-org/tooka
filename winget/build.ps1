Param (
    [Parameter(Mandatory = $true)]
    [string]$Version,

    [Parameter(Mandatory = $false)]
    [string]$Token
)

function Get-GitHubFileHash {
    param (
        [Parameter(Mandatory = $true)]
        [string]$FileName,
        [Parameter(Mandatory = $true)]
        [string]$Version
    )

    $url = "https://github.com/Benji377/tooka/releases/download/v$Version/$FileName"
    $tempFile = "$env:TEMP\$FileName"

    Invoke-WebRequest -Uri $url -OutFile $tempFile
    $hash = Get-FileHash -Path $tempFile -Algorithm SHA256
    return $hash.Hash
}

function Write-MetaData {
    param (
        [string]$FileName,
        [string]$Version,
        [string]$HashAmd64Msi,
        [string]$HashAmd64Exe
    )

    $content = Get-Content $FileName -Raw
    $content = $content.Replace('<VERSION>', $Version)
    $content = $content.Replace('<HASH-AMD64>', $HashAmd64Msi)
    $content = $content.Replace('<HASH-EXE>', $HashAmd64Exe)
    $date = Get-Date -Format "yyyy-MM-dd"
    $content = $content.Replace('<DATE>', $date)

    $outputDir = "./$Version"
    if (-not (Test-Path $outputDir)) {
        New-Item -Path $outputDir -ItemType Directory | Out-Null
    }

    $content | Out-File -Encoding UTF8 "$outputDir/$FileName"
}

# Remove 'v' prefix if present
$Version = $Version.TrimStart('v')

# Download and hash installers
$msiName = "tooka_${Version}_x64_en-US.msi"
$exeName = "tooka_${Version}_x64-setup.exe"

$HashAmd64Msi = Get-GitHubFileHash -FileName $msiName -Version $Version
$HashAmd64Exe = Get-GitHubFileHash -FileName $exeName -Version $Version

# Generate updated YAMLs
Get-ChildItem '*.yaml' | ForEach-Object {
    Write-MetaData -FileName $_.Name -Version $Version `
        -HashAmd64Msi $HashAmd64Msi `
        -HashAmd64Exe $HashAmd64Exe
}

# Optional: Submit to winget-pkgs if token is present
if ($Token) {
    # Install latest wingetcreate
    $wingetAppx = "$env:TEMP\wingetcreate.msixbundle"
    Invoke-WebRequest https://aka.ms/wingetcreate/latest/msixbundle -OutFile $wingetAppx
    Add-AppxPackage $wingetAppx

    # Submit manifest folder to winget
    wingetcreate submit --token $Token "$Version"
}
