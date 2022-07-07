param (
	[switch]$Preview,
	[String]$Binding = "STATIC",
	[String]$OutFile = "kv-bulk.json",
	[String]$Base = ".\static\"
)
$BaseDir = (Get-Item -Path $Base)

# Remove outfile if it exists
if (Test-Path $OutFile) {
	Write-Host "Deleting existing $OutFile"
	Remove-Item $OutFile
}

$BinaryExtensions = ".jpg",".ttf",".woff",".woff2",".eot"
Write-Host "Uploading files from directory $BaseDir"
Add-Content -Path $OutFile -Value "["
$PrefixSize = $BaseDir.FullName.Length
# Get every file in the base directory
foreach ($File in Get-ChildItem -File -Recurse -Path $BaseDir) { 
	$SubPath = $File.FullName.substring($PrefixSize).Replace('\','/')
	$IsBinary = $BinaryExtensions.Contains($File.Extension)
	Write-Host "- $SubPath"
	# This will fail if the file is empty
	if ($File.length) {
		if ($IsBinary) {
			Write-Host "Binary data file. Uploading in bulk."
			Add-Content -Path $OutFile -Value "`t{`n`t`t`"key`": `"$SubPath`","
			$EncodedContents = [convert]::ToBase64String((Get-Content -Path $File.FullName -Encoding byte))
			Add-Content -Path $OutFile -Value "`t`t`"value`": `"$EncodedContents`",`n`t`t`"base64`": true`n`t},"
		} else {
			# Upload text files directly, since it's easier than escaping special characters in strings
			if ($Preview.IsPresent) {
				wrangler kv:key put $SubPath --binding $Binding --path $File.FullName --preview
			} else {
				wrangler kv:key put $SubPath --binding $Binding --path $File.FullName
			}
		}
	} else {
		Write-Host "File length is 0. Skipping."
	}
}
# I'm lazy, so add a superfluous value rather than remove the comma from the last key
Add-Content -Path $OutFile -Value "`t{`n`t`t`"key`": `"test`",`n`t`t`"value`": `"test`"`n`t}`n]"

# Upload files in bulk
Write-Host "Uploading bulk binary files:"
if ($Preview.IsPresent) {
	wrangler kv:bulk put $OutFile --binding $Binding --preview
} else {
	wrangler kv:bulk put $OutFile --binding $Binding
}
