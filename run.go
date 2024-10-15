package main

import (
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"time"
)

type DownloadRequest struct {
	URL string `json:"url"`
}

func sendErrorResponse(w http.ResponseWriter, message string, statusCode int) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(statusCode)
	json.NewEncoder(w).Encode(map[string]string{"error": message})
}

func sendJSONResponse(w http.ResponseWriter, data interface{}) {
    w.Header().Set("Content-Type", "application/json")
    json.NewEncoder(w).Encode(data)
}

func main() {
	fmt.Println("test api")

	mux := http.NewServeMux()

	mux.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		fmt.Fprint(w, "hello world")
	})

	mux.HandleFunc("POST /downloadUrl", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			sendErrorResponse(w, "Method not allowed", http.StatusMethodNotAllowed)
			return
		}

		var req DownloadRequest
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			sendErrorResponse(w, "Invalid request body", http.StatusBadRequest)
			log.Printf("Error decoding req: %v", err)
			return
		}
		
		// generate temp dir
		tempDir, err := os.MkdirTemp("", "yt-dlp-")
		if err != nil {
			sendErrorResponse(w, "Error creating temp dir", http.StatusInternalServerError)
			return
		}
		defer os.RemoveAll(tempDir)

		filename := fmt.Sprintf("%d.mp3", time.Now().UnixNano())
		outputPath := filepath.Join(tempDir, filename)

		cmd := exec.Command("yt-dlp", "--format", "140", "-o", outputPath, req.URL)
		output, err := cmd.CombinedOutput()
		if err != nil {
			log.Printf("Error executing yt-dlp: %v\nOutput: %s", err, output)
			sendErrorResponse(w, "Error downloading audio", http.StatusInternalServerError)
			return
		}

		log.Printf("yt-dlp output: %s", output)
		log.Printf("req url: %s", req.URL)

		// TODO: create unique filename and subdir
		// TODO: execute yt-dlp command
		// TODO: pass to rust fundsp service
		sendJSONResponse(w, map[string]string{"received_url": req.URL, "file_path": outputPath})
	})

	if err := http.ListenAndServe("localhost:8080", mux); err != nil {
		fmt.Println(err.Error())
	}
}