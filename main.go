package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"mime/multipart"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"time"

	"yt-sampler/config"
)

type DownloadRequest struct {
	URL					string	`json:"url"`
	SpliceDuration		float64	`json:"spliceDuration"`
	SpliceCount			int		`json:"spliceCount"`
	Reverse				bool	`json:"reverse"`
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

func sendAudioResponse(w http.ResponseWriter, audioData []byte, filename string) {
	w.Header().Set("Content-Type", "audio/mpeg")
	w.Header().Set("Content-Disposition", fmt.Sprintf("attachment; filename=\"%s\"", filename))
	w.Write(audioData)
}

func sendFileToRustService(filePath string, spliceDuration float64, spliceCount int, reverse bool) ([]byte, error) {
	file, err := os.Open(filePath)
	if err != nil {
		return nil, err
	}
	defer file.Close()

	body := &bytes.Buffer{}
	writer := multipart.NewWriter(body)

	part, err := writer.CreateFormFile("file", filepath.Base(filePath))
	if err != nil {
		return nil, err
	}
	_, err = io.Copy(part, file)
	if err != nil {
		return nil, err
	}

	writer.WriteField("spliceDuration", fmt.Sprintf("%f", spliceDuration))
	writer.WriteField("spliceCount", fmt.Sprintf("%d", spliceCount))
	writer.WriteField("reverse", fmt.Sprintf("%t", reverse))

	err = writer.Close()
	if err != nil {
		return nil, err
	}

	req, err := http.NewRequest("POST", "http://localhost:8081/process", body)
	if err != nil {
		return nil, err
	}
	req.Header.Set("Content-Type", writer.FormDataContentType())

	client := &http.Client{}
	resp, err := client.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	return io.ReadAll(resp.Body)
}

func validateRequest(req DownloadRequest) error {
	if req.URL == "" {
		return fmt.Errorf("URL cannot be empty")
	}
	if req.SpliceDuration <= 0 {
		return fmt.Errorf("splice duration must be > 0")
	}
	if req.SpliceCount <= 0 {
		return fmt.Errorf("splice count must be > 0")
	}
	return nil
}

func main() {
	fmt.Println("test api")

	cfg := config.NewConfig()
	mux := http.NewServeMux()

	mux.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		fmt.Fprint(w, "bruh")
	})

	mux.HandleFunc("POST /downloadUrl", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			sendErrorResponse(w, "Method not allowed", http.StatusMethodNotAllowed)
			return
		}

		var req DownloadRequest
		if err := validateRequest(req); err != nil {
			sendErrorResponse(w, err.Error(), http.StatusBadRequest)
			return
		}

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

		// send to rust_audio_service
		processedAudio, err := sendFileToRustService(outputPath, req.SpliceDuration, req.SpliceCount, req.Reverse)
		if err != nil {
			log.Printf("Error with rust_audio_service: %v", err)
			sendErrorResponse(w, "Error processing audio", http.StatusInternalServerError)
			return
		}

		log.Printf("req url: %s", req.URL)

		sendJSONResponse(w, map[string]string{"received_url": req.URL, "file_path": outputPath})
		sendAudioResponse(w, processedAudio, filename)
	})

    serverAddr := fmt.Sprintf("%s:%s", cfg.ServerHost, cfg.ServerPort)
    if err := http.ListenAndServe(serverAddr, mux); err != nil {
        fmt.Println(err.Error())
    }
}
