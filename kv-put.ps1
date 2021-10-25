$BaseDir = (Get-Item -Path ".\static\")
Write-Host "Uploading files from directory $BaseDir"
$PrefixSize = $BaseDir.FullName.Length
# Get every file in the base directory
foreach ($File in Get-ChildItem -File -Recurse -Path $BaseDir) { 
	$SubPath = $File.FullName.substring($PrefixSize)
	Write-Host -NoNewline "$SubPath : "
	# This will fail if the file is empty
	if ($File.length) {
		wrangler kv:key put $SubPath --namespace-id=59ecddf3e5ed4a85bef94a74f6a06272 (Get-Content -Raw $File.FullName)
	} else {
		Write-Host "File length is 0. Skipping."
	}
}