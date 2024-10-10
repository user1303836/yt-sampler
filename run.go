package main

import (
	"encoding/json"
	"fmt"
	"net/http"
)

type DownloadRequest struct {
	URL string `json:"url"`
}

func main() {
	fmt.Println("test api")

	mux := http.NewServeMux()

	mux.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		fmt.Fprint(w, "hello world")
	})

	mux.HandleFunc("POST /downloadUrl", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
			return
		}

		var req DownloadRequest

		err := json.NewDecoder(r.Body).Decode(&req)
		if err != nil {
			http.Error(w, "Invalid request body", http.StatusBadRequest)
			fmt.Println("Err decoding req:", err)
			return
		}

		fmt.Println("req url:", req.URL)

		// TODO: create unique filename and subdir
		// TODO: execute yt-dlp command
		// TODO: pass to rust fundsp service
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(map[string]string{"received_url": req.URL})
	})

	if err := http.ListenAndServe("localhost:8080", mux); err != nil {
		fmt.Println(err.Error())
	}
}