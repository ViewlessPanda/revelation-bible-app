<#
.SYNOPSIS
    Read-only ACL preflight for Revelation builds.

.DESCRIPTION
    Inspects, with Get-Acl only, the folders a release build trusts: the
    repository (sources, checksums, expected hashes, build output), the
    toolchains (cargo, rustup, node, and the VS Build Tools when they sit under
    -DevRoot) and the Git installation that resolves on PATH. It reports every Allow ACE that grants
    write-type rights to a principal other than the current user, SYSTEM,
    Administrators or TrustedInstaller (for example BUILTIN\Users,
    NT AUTHORITY\Authenticated Users, Everyone, INTERACTIVE, service accounts
    or another user's SID), inherited ACEs included, and any object whose
    owner is such a principal (an owner can rewrite the ACL at any time).

    Write-type rights: WriteData/AddFile, AppendData/AddSubdirectory,
    DeleteSubdirectoriesAndFiles, Delete, ChangePermissions, TakeOwnership,
    GENERIC_WRITE and GENERIC_ALL (so Write, Modify and FullControl count).

    CREATOR OWNER (S-1-3-0) is not a finding. It is a template, not a
    principal: no token ever contains that SID, so the ACE grants nothing on
    the object it sits on. When someone creates a child object, Windows
    replaces it with an ACE for the creator on that new child. A creator must
    already have create rights on the folder, and that grant is what this
    script reports. Children made by an untrusted creator are caught by the
    owner and explicit-ACE checks. OWNER RIGHTS (S-1-3-4) is treated as the
    object's owner.

    THIS SCRIPT NEVER CHANGES AN ACL. For each finding it prints the icacls
    commands the owner can review and run in an elevated shell.

.PARAMETER Release
    Exit 1 if there is any finding or any path whose ACL could not be read.
    Setting the environment variable REVELATION_ALLOW_UNSAFE_ACL=1 downgrades
    this to a warning (exit 0) with a loud banner. That override is for local
    demo builds only, never for a build anyone else will run.

.PARAMETER Path
    Check only these paths instead of the built-in list (used by the tests).

.EXAMPLE
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts\preflight-acl.ps1
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts\preflight-acl.ps1 -Release
#>
[CmdletBinding()]
param(
    [switch]$Release,
    [string[]]$Path,
    [string]$RepoRoot,
    [string]$DevRoot = $env:REVELATION_TOOLS_HOME
)

Set-StrictMode -Version 2
$ErrorActionPreference = 'Stop'

$OverrideVar = 'REVELATION_ALLOW_UNSAFE_ACL'
$SidType = [System.Security.Principal.SecurityIdentifier]

$CurrentSid = [System.Security.Principal.WindowsIdentity]::GetCurrent().User.Value
$SidSystem = 'S-1-5-18'
$SidAdmins = 'S-1-5-32-544'
$SidTrustedInstaller = 'S-1-5-80-956008885-3418522649-1831038044-1853292631-2271478464'
$SidCreatorOwner = 'S-1-3-0'
$SidOwnerRights = 'S-1-3-4'
$Trusted = @($CurrentSid, $SidSystem, $SidAdmins, $SidTrustedInstaller)

# Access-mask bits that let a principal change, replace or delete an object,
# or change who may do so.
$WriteBits = [ordered]@{
    'WriteData/AddFile'            = 0x2
    'AppendData/AddSubdirectory'   = 0x4
    'DeleteSubdirectoriesAndFiles' = 0x40
    'Delete'                       = 0x10000
    'ChangePermissions'            = 0x40000
    'TakeOwnership'                = 0x80000
    'GENERIC_ALL'                  = 0x10000000
    'GENERIC_WRITE'                = 0x40000000
}
$WriteMask = 0
foreach ($v in $WriteBits.Values) { $WriteMask = $WriteMask -bor $v }

if (-not $RepoRoot) { $RepoRoot = Split-Path -Parent $PSScriptRoot }

function Get-FullPath([string]$p) {
    $full = $PSCmdlet.GetUnresolvedProviderPathFromPSPath($p)
    if ($full.Length -gt 3) { $full = $full.TrimEnd('\') }
    return $full
}

function Test-Under([string]$child, [string]$root) {
    if ($child -ieq $root) { return $true }
    return $child.StartsWith($root.TrimEnd('\') + '\', [System.StringComparison]::OrdinalIgnoreCase)
}

function Get-SidName([string]$sid) {
    try {
        return (New-Object System.Security.Principal.SecurityIdentifier $sid).Translate([System.Security.Principal.NTAccount]).Value
    } catch {
        return $sid
    }
}

function Get-WriteNames([int]$mask) {
    $names = @()
    foreach ($k in $WriteBits.Keys) {
        if (($mask -band $WriteBits[$k]) -ne 0) { $names += $k }
    }
    return ($names -join ', ')
}

# ---------------------------------------------------------------- targets
$targets = New-Object System.Collections.Generic.List[object]
function Add-Target([string]$p, [string]$group, [string]$fixRoot) {
    if (-not $p) { return }
    $full = Get-FullPath $p
    foreach ($t in $targets) { if ($t.Path -ieq $full) { return } }
    if (-not $fixRoot) {
        if (Test-Path -LiteralPath $full -PathType Leaf) { $fixRoot = [System.IO.Path]::GetDirectoryName($full) }
        else { $fixRoot = $full }
    }
    $targets.Add([pscustomobject]@{ Path = $full; Group = $group; FixRoot = (Get-FullPath $fixRoot) })
}

function Get-ToolFixRoot([string]$p) {
    $full = Get-FullPath $p
    if ($DevRoot -and (Test-Under $full (Get-FullPath $DevRoot))) { return $DevRoot }
    return $null
}

$notes = New-Object System.Collections.Generic.List[string]

if ($Path) {
    foreach ($p in $Path) { Add-Target $p 'custom' $null }
} else {
    $repo = Get-FullPath $RepoRoot
    foreach ($rel in @('', 'sources\checksums', 'pipeline\tests\expected', 'pipeline\out',
                       'target', 'target\release', 'target\release\revelation.exe',
                       'target\release\content', 'target\release\content\kjv.sqlite',
                       'target\release\content\content.sqlite')) {
        Add-Target (Join-Path $repo $rel) 'repository' $repo
    }

    $cargoHome = $env:CARGO_HOME
    if (-not $cargoHome -and $DevRoot) { $cargoHome = Join-Path $DevRoot 'cargo' }
    if (-not $cargoHome) { $cargoHome = Join-Path $env:USERPROFILE '.cargo' }
    $rustupHome = $env:RUSTUP_HOME
    if (-not $rustupHome -and $DevRoot) { $rustupHome = Join-Path $DevRoot 'rustup' }
    if (-not $rustupHome) { $rustupHome = Join-Path $env:USERPROFILE '.rustup' }
    $toolDirs = @((Join-Path $cargoHome 'bin'), $rustupHome)
    if ($DevRoot) {
        $nodeParent = Join-Path $DevRoot 'node'
        if (Test-Path -LiteralPath $nodeParent) {
            $toolDirs += @(Get-ChildItem -LiteralPath $nodeParent -Directory | ForEach-Object { $_.FullName })
        }
        $toolDirs += (Join-Path $DevRoot 'VS\BuildTools2022')
    }
    foreach ($exe in @('cargo.exe', 'rustc.exe', 'node.exe')) {
        $cmd = Get-Command $exe -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($cmd) { $toolDirs += $cmd.Source } else { $notes.Add("$exe does not resolve on PATH") }
    }
    foreach ($d in $toolDirs) { Add-Target $d 'toolchain' (Get-ToolFixRoot $d) }

    $git = Get-Command git.exe -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($git) {
        $gitDir = [System.IO.Path]::GetDirectoryName($git.Source)
        $gitRoot = $gitDir
        if ((Split-Path -Leaf $gitRoot) -in @('cmd', 'bin')) { $gitRoot = [System.IO.Path]::GetDirectoryName($gitRoot) }
        if ((Split-Path -Leaf $gitRoot) -in @('mingw64', 'usr')) { $gitRoot = [System.IO.Path]::GetDirectoryName($gitRoot) }
        $systemPath = (Get-ItemProperty 'HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Environment').Path -split ';' |
            Where-Object { $_ } | ForEach-Object { [Environment]::ExpandEnvironmentVariables($_).TrimEnd('\') }
        $onSystemPath = @($systemPath | Where-Object { $_ -ieq $gitDir }).Count -gt 0
        $where = 'user PATH'
        if ($onSystemPath) { $where = 'SYSTEM PATH (searched by elevated shells too)' }
        $notes.Add("git.exe resolves to $($git.Source); $gitDir is on the $where")
        foreach ($p in @($gitRoot, $gitDir, $git.Source)) { Add-Target $p 'git' $gitRoot }
    } else {
        $notes.Add('git.exe does not resolve on PATH')
    }
}

# ---------------------------------------------------------------- scan
$findings = New-Object System.Collections.Generic.List[object]
$unreadable = New-Object System.Collections.Generic.List[string]
$missing = New-Object System.Collections.Generic.List[string]
$checked = 0

foreach ($t in $targets) {
    if (-not (Test-Path -LiteralPath $t.Path)) { $missing.Add($t.Path); continue }
    try {
        $acl = Get-Acl -LiteralPath $t.Path
        $rules = $acl.GetAccessRules($true, $true, $SidType)
        $ownerSid = $acl.GetOwner($SidType).Value
    } catch {
        $unreadable.Add("$($t.Path): $($_.Exception.Message)")
        continue
    }
    $checked++
    $ownerTrusted = $Trusted -contains $ownerSid
    if (-not $ownerTrusted) {
        $findings.Add([pscustomobject]@{
            Target = $t; Kind = 'owner'; Sid = $ownerSid; Name = (Get-SidName $ownerSid)
            Rights = 'owner (can rewrite the ACL)'; Inherited = $false; Applies = 'this object'
        })
    }
    foreach ($r in $rules) {
        if ($r.AccessControlType -ne [System.Security.AccessControl.AccessControlType]::Allow) { continue }
        $mask = [int]$r.FileSystemRights
        if (($mask -band $WriteMask) -eq 0) { continue }
        $sid = $r.IdentityReference.Value
        if ($sid -eq $SidCreatorOwner) { continue }   # template only; see the help text above
        if ($sid -eq $SidOwnerRights -and $ownerTrusted) { continue }
        if ($Trusted -contains $sid) { continue }
        $inheritOnly = ($r.PropagationFlags -band [System.Security.AccessControl.PropagationFlags]::InheritOnly) -ne 0
        $inheritable = $r.InheritanceFlags -ne [System.Security.AccessControl.InheritanceFlags]::None
        if ($inheritOnly) { $applies = 'contents only (inherited by files and subfolders)' }
        elseif ($inheritable) { $applies = 'this folder and its contents' }
        else { $applies = 'this object' }
        $findings.Add([pscustomobject]@{
            Target = $t; Kind = 'ace'; Sid = $sid; Name = (Get-SidName $sid)
            Rights = (Get-WriteNames $mask); Inherited = $r.IsInherited; Applies = $applies
        })
    }
}

# ---------------------------------------------------------------- fix plan
# Walks up from $p to the nearest ancestor where $sid has an explicit
# (non-inherited) Allow ACE: the place the inherited grant comes from.
function Find-AceSource([string]$p, [string]$sid) {
    $cur = $p
    while ($true) {
        $parent = [System.IO.Path]::GetDirectoryName($cur)
        if (-not $parent) { return $null }
        try { $pacl = Get-Acl -LiteralPath $parent } catch { return $null }
        $mine = @($pacl.GetAccessRules($true, $true, $SidType) | Where-Object {
            $_.IdentityReference.Value -eq $sid -and $_.AccessControlType -eq 'Allow' })
        if ($mine.Count -eq 0) { return $null }
        if (@($mine | Where-Object { -not $_.IsInherited }).Count -gt 0) { return $parent }
        $cur = $parent
    }
}

$plan = [ordered]@{}
function Get-PlanEntry([string]$p) {
    if (-not $plan.Contains($p)) {
        $plan[$p] = [pscustomobject]@{
            Path = $p; IsDir = (Test-Path -LiteralPath $p -PathType Container)
            Break = $false; Remove = (New-Object System.Collections.Generic.List[string]); SetOwner = $false
        }
    }
    return $plan[$p]
}

foreach ($f in $findings) {
    if ($f.Kind -eq 'owner') { (Get-PlanEntry $f.Target.Path).SetOwner = $true; continue }
    if (-not $f.Inherited) {
        $e = Get-PlanEntry $f.Target.Path
    } else {
        $source = Find-AceSource $f.Target.Path $f.Sid
        if ($source -and (Test-Under $source $f.Target.FixRoot)) {
            $e = Get-PlanEntry $source
        } else {
            # The grant comes from above the folder we want to protect (for
            # example a drive root): stop inheriting there instead of
            # changing the whole drive.
            $e = Get-PlanEntry $f.Target.FixRoot
            $e.Break = $true
        }
    }
    if (-not $e.Remove.Contains($f.Sid)) { $e.Remove.Add($f.Sid) }
}

# ---------------------------------------------------------------- report
$mode = 'warn'
if ($Release) { $mode = 'release (-Release)' }
Write-Host "Revelation ACL preflight (M0-F1). Read-only: this script never changes an ACL."
Write-Host "Mode: $mode. Current user: $(Get-SidName $CurrentSid) ($CurrentSid)."
Write-Host "Trusted principals: current user, SYSTEM, Administrators, TrustedInstaller (CREATOR OWNER is a template, not a grant)."
foreach ($n in $notes) { Write-Host "Note: $n" }
Write-Host "Checked $checked path(s); $($missing.Count) not present (skipped)."
foreach ($m in $missing) { Write-Host "  not present: $m" }
Write-Host ''

if ($findings.Count -eq 0 -and $unreadable.Count -eq 0) {
    Write-Host 'OK: no non-admin principal can write to the checked paths.' -ForegroundColor Green
    exit 0
}

if ($findings.Count -gt 0) {
    Write-Host "FINDINGS ($($findings.Count)): these principals can modify what the build trusts." -ForegroundColor Yellow
    foreach ($group in ($findings | Group-Object { $_.Target.Path })) {
        Write-Host "  $($group.Name)"
        foreach ($f in $group.Group) {
            $how = 'explicit'
            if ($f.Inherited) { $how = 'inherited' }
            if ($f.Kind -eq 'owner') { $how = 'owner' }
            Write-Host "    - $($f.Name) [$($f.Sid)], $how, $($f.Applies): $($f.Rights)"
        }
    }
    Write-Host ''
}
foreach ($u in $unreadable) { Write-Host "UNVERIFIABLE (ACL could not be read): $u" -ForegroundColor Yellow }

if ($plan.Count -gt 0) {
    Write-Host 'SUGGESTED FIX. Not run by this script. REVIEW EACH LINE BEFORE RUNNING IT,' -ForegroundColor Yellow
    Write-Host 'in an elevated PowerShell (Run as administrator). The first line of each block gives' -ForegroundColor Yellow
    Write-Host 'you (the current user) explicit full control, because under UAC your everyday shell' -ForegroundColor Yellow
    Write-Host 'does not use the Administrators group; without it you lose write access to your own' -ForegroundColor Yellow
    Write-Host 'files. SYSTEM and Administrators keep their access. /inheritance:d copies the inherited' -ForegroundColor Yellow
    Write-Host 'entries onto the folder before the bad grant is removed, so nothing else changes.' -ForegroundColor Yellow
    Write-Host ''
    foreach ($e in $plan.Values) {
        $q = '"' + $e.Path + '"'
        Write-Host "# $($e.Path)"
        if ($e.IsDir) { Write-Host "icacls $q /grant:r `"*$($CurrentSid):(OI)(CI)F`"" }
        else { Write-Host "icacls $q /grant:r `"*$($CurrentSid):F`"" }
        if ($e.Break) { Write-Host "icacls $q /inheritance:d" }
        if ($e.Remove.Count -gt 0) {
            $sids = ($e.Remove | ForEach-Object { "`"*$_`"" }) -join ' '
            $who = ($e.Remove | ForEach-Object { Get-SidName $_ }) -join ', '
            Write-Host "icacls $q /remove:g $sids    # removes every grant to: $who"
        }
        if ($e.SetOwner) { Write-Host "icacls $q /setowner `"*$CurrentSid`"" }
        Write-Host ''
    }
    Write-Host 'Files or folders below these that carry their own explicit grants are not changed by'
    Write-Host 'the lines above; run the preflight again afterwards and confirm it reports OK.'
    Write-Host 'Stricter option for a Git directory on the SYSTEM PATH: leave out the /grant:r line'
    Write-Host '(only administrators can then write there), or reinstall Git under C:\Program Files.'
    Write-Host ''
}

if (-not $Release) {
    Write-Host 'Warn mode: exit 0. A release build (npm run release) fails until this is fixed.' -ForegroundColor Yellow
    exit 0
}

if ($env:REVELATION_ALLOW_UNSAFE_ACL -eq '1') {
    $bar = '!' * 78
    Write-Host $bar -ForegroundColor Red
    Write-Host "!!  $OverrideVar=1: RELEASE ACL CHECK OVERRIDDEN. LOCAL DEMO BUILD ONLY." -ForegroundColor Red
    Write-Host '!!  The toolchain or sources may have been modified by another account.' -ForegroundColor Red
    Write-Host '!!  Do not give this build to anyone or treat it as a release.' -ForegroundColor Red
    Write-Host $bar -ForegroundColor Red
    exit 0
}

Write-Host "RELEASE BLOCKED (M0-F1): fix the findings above, or for a local demo build only set $OverrideVar=1." -ForegroundColor Red
exit 1
