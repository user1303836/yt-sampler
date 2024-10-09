package main

import "net/http"

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
