<#
.SYNOPSIS
    Tests for scripts\preflight-acl.ps1 against a throwaway folder.

.DESCRIPTION
    Creates a scratch folder under -Root (default %TEMP%),
    changes the ACL of THAT FOLDER ONLY with icacls, runs the preflight against
    it (-Path) in each state, checks the exit code and output, then deletes
    the folder. It also checks that the preflight itself leaves the ACL
    unchanged (same SDDL before and after) and that the icacls fix it prints
    really clears the finding.

    powershell -NoProfile -ExecutionPolicy Bypass -File scripts\test-preflight-acl.ps1
#>
param([string]$Root = $env:TEMP)
Set-StrictMode -Version 2
$ErrorActionPreference = 'Stop'

$preflight = Join-Path $PSScriptRoot 'preflight-acl.ps1'
$scratch = Join-Path $Root ('revelation-acl-test-' + [guid]::NewGuid().ToString('N'))
$failures = 0
$passes = 0

function Invoke-Preflight([switch]$Release, [string]$Override) {
    $args2 = @('-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass', '-File', $preflight, '-Path', $scratch)
    if ($Release) { $args2 += '-Release' }
    $old = $env:REVELATION_ALLOW_UNSAFE_ACL
    if ($Override) { $env:REVELATION_ALLOW_UNSAFE_ACL = $Override } else { Remove-Item Env:REVELATION_ALLOW_UNSAFE_ACL -ErrorAction SilentlyContinue }
    try {
        $out = & powershell.exe @args2 | Out-String
        $code = $LASTEXITCODE
    } finally {
        if ($null -ne $old) { $env:REVELATION_ALLOW_UNSAFE_ACL = $old } else { Remove-Item Env:REVELATION_ALLOW_UNSAFE_ACL -ErrorAction SilentlyContinue }
    }
    return [pscustomobject]@{ Exit = $code; Out = $out }
}

function Check([string]$name, [bool]$ok, [string]$detail) {
    if ($ok) { $script:passes++; Write-Host "PASS  $name" }
    else { $script:failures++; Write-Host "FAIL  $name`n$detail" -ForegroundColor Red }
}

# icacls on the scratch folder only (guarded).
function Set-ScratchAcl([string[]]$icaclsArgs) {
    if (-not ((Split-Path -Leaf $scratch) -like 'revelation-acl-test-*')) { throw 'refusing to touch a non-scratch path' }
    $null = & icacls.exe $scratch @icaclsArgs
    if ($LASTEXITCODE -ne 0) { throw "icacls $icaclsArgs failed" }
}

function Get-Sddl { (Get-Acl -LiteralPath $scratch).Sddl }

# Runs the icacls lines the preflight printed (for the scratch folder only).
function Invoke-PrintedFix([string]$out) {
    $lines = @($out -split "`r?`n" | Where-Object { $_ -like 'icacls *' })
    foreach ($l in $lines) {
        if ($l -notlike ('icacls "' + $scratch + '" *')) { throw "printed fix targets another path: $l" }
        $cmd = ($l -split '    #')[0]
        Invoke-Expression ($cmd -replace '^icacls ', '& icacls.exe ') | Out-Null
        if ($LASTEXITCODE -ne 0) { throw "fix line failed: $l" }
    }
    return $lines.Count
}

New-Item -ItemType Directory -Path $scratch | Out-Null
Set-Content -LiteralPath (Join-Path $scratch 'tool.exe') -Value 'not a program' -Encoding ascii
Write-Host "Scratch folder: $scratch"
Write-Host "Inherited ACL:"; & icacls.exe $scratch | Select-Object -SkipLast 2 | ForEach-Object { Write-Host "  $_" }
$inheritsAuthUsers = (& icacls.exe $scratch | Out-String) -match 'Authenticated Users:\(I\)'

try {
    # 1. as created: whatever %TEMP% hands down. On some machines that
    #    is Authenticated Users:(M) inherited from a drive root.
    if ($inheritsAuthUsers) {
        $before = Get-Sddl
        $r = Invoke-Preflight
        Check 'inherited Authenticated Users Modify: warn mode reports it, exit 0' (($r.Exit -eq 0) -and ($r.Out -match 'S-1-5-11\], inherited')) $r.Out
        Check 'the preflight does not change the ACL (SDDL unchanged)' ((Get-Sddl) -eq $before) ''
        $r = Invoke-Preflight -Release
        Check 'inherited Authenticated Users Modify: -Release exits 1' (($r.Exit -eq 1) -and ($r.Out -match 'RELEASE BLOCKED')) $r.Out
        Check 'fix for an inherited grant stops inheritance at the folder' ($r.Out -match '(?m)^icacls .* /inheritance:d') $r.Out
        $r2 = Invoke-Preflight -Release -Override '1'
        Check 'override=1 downgrades -Release to exit 0 with the banner' (($r2.Exit -eq 0) -and ($r2.Out -match 'OVERRIDDEN')) $r2.Out
        $r3 = Invoke-Preflight -Release -Override 'true'
        Check 'override with any value other than 1 is ignored' ($r3.Exit -eq 1) $r3.Out
        $n = Invoke-PrintedFix $r.Out
        $r = Invoke-Preflight -Release
        Check "running the $n printed icacls line(s) clears it: -Release exits 0" (($r.Exit -eq 0) -and ($r.Out -match 'OK: no non-admin')) $r.Out
        $child = & icacls.exe (Join-Path $scratch 'tool.exe') | Out-String
        Check 'the fix reached the file inside (no Authenticated Users on tool.exe)' ($child -notmatch 'Authenticated Users') $child
        Check 'the fix kept the current user, SYSTEM and Administrators' (($child -match [regex]::Escape($env:USERNAME)) -and ($child -match 'SYSTEM') -and ($child -match 'Administrators')) $child
    } else {
        Write-Host 'SKIP  inherited cases: -Root does not hand down an Authenticated Users grant here'
        Set-ScratchAcl @('/inheritance:d')
    }

    # 2. clean, protected folder: no finding.
    Set-ScratchAcl @('/inheritance:r', '/grant:r', "*$([Security.Principal.WindowsIdentity]::GetCurrent().User.Value):(OI)(CI)F", '*S-1-5-18:(OI)(CI)F', '*S-1-5-32-544:(OI)(CI)F')
    $r = Invoke-Preflight -Release
    Check 'only current user, SYSTEM, Administrators: -Release exits 0' (($r.Exit -eq 0) -and ($r.Out -match 'OK: no non-admin')) $r.Out

    # 3. explicit Authenticated Users Modify ACE.
    Set-ScratchAcl @('/grant', '*S-1-5-11:(OI)(CI)M')
    $r = Invoke-Preflight
    Check 'explicit Authenticated Users Modify: reported as explicit, warn exit 0' (($r.Exit -eq 0) -and ($r.Out -match 'S-1-5-11\], explicit')) $r.Out
    $r = Invoke-Preflight -Release
    Check 'explicit Authenticated Users Modify: -Release exits 1' ($r.Exit -eq 1) $r.Out
    Check 'fix for an explicit grant removes it in place (no /inheritance:d)' (($r.Out -match '/remove:g "\*S-1-5-11"') -and ($r.Out -notmatch '(?m)^icacls .* /inheritance:d')) $r.Out
    $n = Invoke-PrintedFix $r.Out
    $r = Invoke-Preflight -Release
    Check "running the $n printed icacls line(s) clears it: -Release exits 0" ($r.Exit -eq 0) $r.Out

    # 4. read-only grants and CREATOR OWNER are not findings.
    Set-ScratchAcl @('/grant', '*S-1-5-32-545:(OI)(CI)RX', '*S-1-1-0:(RX)', '*S-1-3-0:(OI)(CI)(IO)F')
    $r = Invoke-Preflight -Release
    Check 'Users/Everyone read-only and CREATOR OWNER full control: no finding' (($r.Exit -eq 0) -and ($r.Out -match 'OK: no non-admin')) $r.Out

    # 5. each write-type right on its own, for several principals.
    $cases = @(
        @('Everyone write', '*S-1-1-0:(W)', 'S-1-1-0'),
        @('INTERACTIVE append only (AD)', '*S-1-5-4:(AD)', 'S-1-5-4'),
        @('LOCAL SERVICE delete only (D)', '*S-1-5-19:(D)', 'S-1-5-19'),
        @('Users change permissions (WDAC)', '*S-1-5-32-545:(WDAC)', 'S-1-5-32-545'),
        @('another account take ownership (WO)', '*S-1-5-20:(WO)', 'S-1-5-20'),
        @('Users contents-only generic write (OI)(IO)(GW)', '*S-1-5-32-545:(OI)(IO)(GW)', 'S-1-5-32-545')
    )
    foreach ($c in $cases) {
        Set-ScratchAcl @('/grant', $c[1])
        $r = Invoke-Preflight -Release
        Check "$($c[0]): reported, -Release exits 1" (($r.Exit -eq 1) -and ($r.Out -match [regex]::Escape($c[2]) + '\]')) $r.Out
        Set-ScratchAcl @('/remove:g', "*$($c[2])")
        if ($c[2] -eq 'S-1-5-32-545') { Set-ScratchAcl @('/grant', '*S-1-5-32-545:(OI)(CI)RX') }
    }

    # 6. a path that does not exist is skipped, not a finding.
    $gone = $scratch + '-missing'
    $out = & powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File $preflight -Path $gone -Release | Out-String
    Check 'missing path: skipped, exit 0' (($LASTEXITCODE -eq 0) -and ($out -match 'not present')) $out
} finally {
    Remove-Item -LiteralPath $scratch -Recurse -Force
}

Write-Host ''
Write-Host "$passes passed, $failures failed"
if ($failures -gt 0) { exit 1 }
exit 0
