Set-Location -Path (Split-Path -Parent $MyInvocation.MyCommand.Path) -ErrorAction Stop

if ($args.Count -eq 0) {
    Write-Host "Available:"
    if (Test-Path "dev_tools/src/bin") {
        $files = Get-ChildItem -Path "dev_tools/src/bin/*.rs"
        foreach ($file in $files) {
            if ($file -is [System.IO.FileInfo]) {
                Write-Host $file.BaseName
            }
        }
    } else {
        Write-Host "Warning: dev_tools/src/bin directory does not exist"
    }
    if (Test-Path "dev_tools/scripts") {
        $scripts = Get-ChildItem -Path "dev_tools/scripts/*.ps1", "dev_tools/scripts/*.py"
        foreach ($script in $scripts) {
            if ($script -is [System.IO.FileInfo]) {
                Write-Host $script.BaseName
            }
        }
    } else {
        Write-Host "Warning: dev_tools/scripts directory does not exist"
    }
    exit 1
}

$target_name = $args[0]
$script_file_ps1 = "dev_tools/scripts/${target_name}.ps1"
$script_file_py = "dev_tools/scripts/${target_name}.py"
$rust_file = "dev_tools/src/bin/${target_name}.rs"

if (Test-Path $script_file_ps1) {
    & $script_file_ps1
} elseif (Test-Path $script_file_py) {
    python $script_file_py
} elseif (Test-Path $rust_file) {
    cargo run --manifest-path dev_tools/Cargo.toml --bin $target_name --quiet
} else {
    Write-Host "Error: target '$target_name' does not exist as a script or Rust program"
    exit 1
}
