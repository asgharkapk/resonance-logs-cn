# One-off generator for the placeholder voice-preset reference WAVs under
# src-tauri/voice-presets/. Not part of the build and not bundled into the
# app; run manually to (re)create the placeholders or to try out a
# different placeholder tone while the real, licensed reference recordings
# are still pending. See src-tauri/voice-presets/LICENSE.md for the
# licensing status of whatever files currently live in that directory.
#
# Produces a synthetic multi-harmonic tone (not real speech) that is a
# valid RIFF/WAVE PCM file satisfying src-tauri/src/voice/audio.rs's
# validation (mono/stereo PCM, 1-120s duration, <=50MB). It is good enough
# to exercise the speaker-clone pipeline end to end, but the resulting
# "voice" will sound like a tone, not a person - replace before shipping.
param(
    [int]$SampleRate = 16000,
    [double]$DurationSec = 3.0
)

function New-PlaceholderWav {
    param(
        [Parameter(Mandatory = $true)][string]$Path,
        [Parameter(Mandatory = $true)][int]$SampleRate,
        [Parameter(Mandatory = $true)][double]$DurationSec,
        [Parameter(Mandatory = $true)][double]$FrequencyHz
    )

    $channels = 1
    $bitsPerSample = 16
    $numSamples = [int]($SampleRate * $DurationSec)
    $byteRate = $SampleRate * $channels * $bitsPerSample / 8
    $blockAlign = $channels * $bitsPerSample / 8
    $dataSize = $numSamples * $blockAlign

    $stream = [System.IO.File]::Create($Path)
    try {
        $writer = New-Object System.IO.BinaryWriter($stream)

        $writer.Write([System.Text.Encoding]::ASCII.GetBytes("RIFF"))
        $writer.Write([UInt32](36 + $dataSize))
        $writer.Write([System.Text.Encoding]::ASCII.GetBytes("WAVE"))

        $writer.Write([System.Text.Encoding]::ASCII.GetBytes("fmt "))
        $writer.Write([UInt32]16)
        $writer.Write([UInt16]1) # PCM
        $writer.Write([UInt16]$channels)
        $writer.Write([UInt32]$SampleRate)
        $writer.Write([UInt32]$byteRate)
        $writer.Write([UInt16]$blockAlign)
        $writer.Write([UInt16]$bitsPerSample)

        $writer.Write([System.Text.Encoding]::ASCII.GetBytes("data"))
        $writer.Write([UInt32]$dataSize)

        $amplitude = 0.2 * [Int16]::MaxValue
        for ($i = 0; $i -lt $numSamples; $i++) {
            $t = $i / $SampleRate
            $envelope = 0.6 + 0.4 * [Math]::Sin(2 * [Math]::PI * 0.5 * $t)
            $sampleValue = $amplitude * $envelope * (
                0.6 * [Math]::Sin(2 * [Math]::PI * $FrequencyHz * $t) +
                0.3 * [Math]::Sin(2 * [Math]::PI * $FrequencyHz * 2 * $t) +
                0.1 * [Math]::Sin(2 * [Math]::PI * $FrequencyHz * 3 * $t)
            )
            $writer.Write([Int16]([Math]::Round($sampleValue)))
        }

        $writer.Flush()
    }
    finally {
        $stream.Close()
    }
}

$repoRoot = Split-Path -Parent $PSScriptRoot
$presetsDir = Join-Path $repoRoot "src-tauri\voice-presets"
New-PlaceholderWav -Path (Join-Path $presetsDir "zh-CN.wav") -SampleRate $SampleRate -DurationSec $DurationSec -FrequencyHz 196.0
New-PlaceholderWav -Path (Join-Path $presetsDir "en-US.wav") -SampleRate $SampleRate -DurationSec $DurationSec -FrequencyHz 220.0
New-PlaceholderWav -Path (Join-Path $presetsDir "ja-JP.wav") -SampleRate $SampleRate -DurationSec $DurationSec -FrequencyHz 246.94
Write-Output "generated placeholder WAVs in $presetsDir"
