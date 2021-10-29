for file in $(ls *.jpg *.ttf *.woff *.woff2 *.eot)
do
	base64 "$file" > "$file".b64
	mv "$file".b64 "$file"
done
