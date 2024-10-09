package main

import (
	"fmt"
	"os/exec"
)

func main() {
	cmd := exec.Command("yt-dlp", "-x", "--audio-format", "mp3", "https://www.youtube.com/watch?v=dQw4w9WgXcQ")
	
	output, err := cmd.CombinedOutput()
	if err != nil {
		fmt.Println("Err: ", err)
		return
	}

	fmt.Println("Cmd output: ", string(output))
}