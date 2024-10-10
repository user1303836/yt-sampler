package main

import "net/http"

type DownloadRequest struct {
	URL string `json:"url"`
}

type DownloadResponse struct {
	DownloadID string `json:"download_id"`
	Status     string `json:"status"`
}

func main() {
	http.HandleFunc("/api/download", handleDownloadRequest)
	http.HandleFunc("/download/", serveDownload)
}

func handleDownloadRequest(w http.ResponseWriter, r *http.Request) {
	return
}

func serveDownload(w http.ResponseWriter, r *http.Request) {
	return
}
