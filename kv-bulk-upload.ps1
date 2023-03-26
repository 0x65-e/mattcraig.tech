param (
	[switch]$Preview,
	[String]$Binding = "STATIC",
	[String]$OutFile = "kv-bulk.json",
	[String]$Base = ".\static\",
	[switch]$SkipBinary,
	[int]$LimitSize = 1
)
$BaseDir = (Get-Item -Path $Base)

# Remove outfile if it exists
if ((Test-Path $OutFile) -and -not $SkipBinary.IsPresent) {
	Write-Host "Deleting existing $OutFile"
	Remove-Item $OutFile
}

$BinaryExtensions = ".jpg",".jpeg",".png",".gif",".webp",".ttf",".woff",".woff2",".eot",".pdf",".wav",".mp3",".mpeg",".mp4",".otf",".zip",".7z"
Write-Host "Uploading files from directory $BaseDir to keystore $Binding"
if ($Preview.IsPresent) {
	Write-Host "Using preview keystore"
}
Add-Content -Path $OutFile -Value "["
$PrefixSize = $BaseDir.FullName.Length
# Get every file in the base directory
foreach ($File in Get-ChildItem -File -Recurse -Path $BaseDir) { 
	$SubPath = $File.FullName.substring($PrefixSize).Replace('\','/')
	$IsBinary = $BinaryExtensions.Contains($File.Extension)
	Write-Host "- $SubPath"
	# This will fail if the file is empty
	if ($File.length) {
		if (($LimitSize -ne 0) -and (($File.length / 1MB) -gt $LimitSize)) {
			Write-Warning "Skipping large file."
			continue
		}
		if ($IsBinary) {
			if ($SkipBinary.IsPresent) {
				Write-Warning "Skipping binary data file."
			} else {
				Write-Host "Binary data file. Uploading in bulk."
				Add-Content -Path $OutFile -Value "`t{`n`t`t`"key`": `"$SubPath`","
				$EncodedContents = [convert]::ToBase64String((Get-Content -Path $File.FullName -AsByteStream))
				Add-Content -Path $OutFile -Value "`t`t`"value`": `"$EncodedContents`",`n`t`t`"base64`": true`n`t},"
			}
		} else {
			# Upload text files directly, since it's easier than escaping special characters in strings
			if ($Preview.IsPresent) {
				npx wrangler kv:key put $SubPath --binding $Binding --path $File.FullName --preview
			} else {
				npx wrangler kv:key put $SubPath --binding $Binding --path $File.FullName --preview false
			}
		}
	} else {
		Write-Host "File length is 0. Skipping."
	}
}
# Add a superfluous value rather than remove the comma from the last key
Add-Content -Path $OutFile -Value "`t{`n`t`t`"key`": `"sentinel`",`n`t`t`"value`": `"active`"`n`t}`n]"

if (-not $SkipBinary.IsPresent) {
	# Upload files in bulk
	Write-Host "Uploading bulk binary files:"
	if ($Preview.IsPresent) {
		npx wrangler kv:bulk put $OutFile --binding $Binding --preview
		npx wrangler kv:key delete sentinel --binding $Binding --preview # Remember to delete sentinel value
	} else {
		npx wrangler kv:bulk put $OutFile --binding $Binding --preview false
		npx wrangler kv:key delete sentinel --binding $Binding --preview false
	}

	# Remove outfile if it exists
	if (Test-Path $OutFile) {
		Write-Host "Deleting $OutFile"
		Remove-Item $OutFile
	}
}
