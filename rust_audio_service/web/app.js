class AudioProcessor {
    constructor() {
        this.apiBase = window.location.origin;
        this.currentFile = null;
        this.currentBlob = null;
        this.initializeElements();
        this.setupEventListeners();
        this.checkAPIHealth();
    }

    initializeElements() {
        // File upload elements
        this.uploadArea = document.getElementById('uploadArea');
        this.audioFile = document.getElementById('audioFile');
        this.fileInfo = document.getElementById('fileInfo');
        
        // Tab elements
        this.tabButtons = document.querySelectorAll('.tab-button');
        this.effectPanels = document.querySelectorAll('.effect-panel');
        
        // Form elements
        this.processButton = document.getElementById('processButton');
        this.progress = document.getElementById('progress');
        this.progressBar = document.querySelector('.progress-bar');
        
        // Result elements
        this.resultsSection = document.getElementById('resultsSection');
        this.resultInfo = document.getElementById('resultInfo');
        this.downloadButton = document.getElementById('downloadButton');
        
        // Error elements
        this.errorSection = document.getElementById('errorSection');
        this.errorMessage = document.getElementById('errorMessage');
        this.retryButton = document.getElementById('retryButton');
        
        // Status
        this.status = document.getElementById('status');
    }

    setupEventListeners() {
        // File upload
        this.uploadArea.addEventListener('click', () => this.audioFile.click());
        this.audioFile.addEventListener('change', (e) => this.handleFileSelect(e));
        
        // Drag and drop
        this.uploadArea.addEventListener('dragover', (e) => this.handleDragOver(e));
        this.uploadArea.addEventListener('dragleave', (e) => this.handleDragLeave(e));
        this.uploadArea.addEventListener('drop', (e) => this.handleDrop(e));
        
        // Tabs
        this.tabButtons.forEach(button => {
            button.addEventListener('click', () => this.switchTab(button.dataset.tab));
        });
        
        // Process button
        this.processButton.addEventListener('click', () => this.processAudio());
        
        // Download button
        this.downloadButton.addEventListener('click', () => this.downloadResults());
        
        // Retry button
        this.retryButton.addEventListener('click', () => this.hideError());
    }

    async checkAPIHealth() {
        try {
            const response = await fetch(`${this.apiBase}/api/v1/health`);
            const data = await response.json();
            this.updateStatus(`API Online - v${data.version}`, 'success');
        } catch (error) {
            this.updateStatus('API Offline', 'error');
            console.error('API health check failed:', error);
        }
    }

    updateStatus(message, type = 'info') {
        this.status.textContent = message;
        this.status.className = `status ${type}`;
    }

    handleFileSelect(event) {
        const file = event.target.files[0];
        if (file) {
            this.handleFile(file);
        }
    }

    handleDragOver(event) {
        event.preventDefault();
        this.uploadArea.classList.add('dragover');
    }

    handleDragLeave(event) {
        event.preventDefault();
        this.uploadArea.classList.remove('dragover');
    }

    handleDrop(event) {
        event.preventDefault();
        this.uploadArea.classList.remove('dragover');
        
        const files = event.dataTransfer.files;
        if (files.length > 0) {
            this.handleFile(files[0]);
        }
    }

    handleFile(file) {
        // Validate file type
        if (!file.name.toLowerCase().endsWith('.wav')) {
            this.showError('Please select a WAV file. Other formats are not currently supported.');
            return;
        }

        // Validate file size (100MB limit)
        const maxSize = 100 * 1024 * 1024; // 100MB
        if (file.size > maxSize) {
            this.showError('File size must be less than 100MB.');
            return;
        }

        this.currentFile = file;
        this.showFileInfo(file);
        this.processButton.disabled = false;
        this.updateStatus('File ready for processing', 'success');
    }

    showFileInfo(file) {
        const sizeInMB = (file.size / (1024 * 1024)).toFixed(2);
        const lastModified = new Date(file.lastModified).toLocaleDateString();
        
        this.fileInfo.innerHTML = `
            <strong>📁 ${file.name}</strong><br>
            Size: ${sizeInMB} MB | Modified: ${lastModified}
        `;
        this.fileInfo.style.display = 'block';
    }

    switchTab(tabName) {
        // Update tab buttons
        this.tabButtons.forEach(button => {
            button.classList.toggle('active', button.dataset.tab === tabName);
        });
        
        // Update panels
        this.effectPanels.forEach(panel => {
            panel.classList.toggle('active', panel.id === `${tabName}-panel`);
        });
    }

    async processAudio() {
        if (!this.currentFile) {
            this.showError('Please select an audio file first.');
            return;
        }

        const activeTab = document.querySelector('.tab-button.active').dataset.tab;
        this.hideError();
        this.hideResults();
        this.showProgress();
        this.processButton.disabled = true;
        this.updateStatus('Processing audio...', 'processing');

        try {
            let result;
            if (activeTab === 'splice') {
                result = await this.processSplice();
            } else if (activeTab === 'normalize') {
                result = await this.processNormalize();
            }

            this.hideProgress();
            this.showResults(result, activeTab);
            this.updateStatus('Processing complete!', 'success');
        } catch (error) {
            this.hideProgress();
            this.showError(error.message);
            this.updateStatus('Processing failed', 'error');
        } finally {
            this.processButton.disabled = false;
        }
    }

    async processSplice() {
        const formData = new FormData();
        formData.append('file', this.currentFile);
        formData.append('spliceDuration', document.getElementById('spliceDuration').value);
        formData.append('spliceCount', document.getElementById('spliceCount').value);
        formData.append('reverse', document.getElementById('reverse').checked);

        const response = await fetch(`${this.apiBase}/api/v1/audio/splice/multipart`, {
            method: 'POST',
            body: formData
        });

        if (!response.ok) {
            const errorData = await response.json().catch(() => null);
            throw new Error(errorData?.error || `HTTP ${response.status}: ${response.statusText}`);
        }

        const blob = await response.blob();
        return {
            blob,
            filename: 'audio_splices.zip',
            type: 'splice',
            duration: document.getElementById('spliceDuration').value,
            count: document.getElementById('spliceCount').value,
            reverse: document.getElementById('reverse').checked
        };
    }

    async processNormalize() {
        const formData = new FormData();
        formData.append('file', this.currentFile);
        formData.append('targetLevel', document.getElementById('targetLevel').value);
        
        const normalizeMode = document.querySelector('input[name="normalizeMode"]:checked').value;
        formData.append('applyToSplices', normalizeMode);

        const response = await fetch(`${this.apiBase}/api/v1/audio/normalize/multipart`, {
            method: 'POST',
            body: formData
        });

        if (!response.ok) {
            const errorData = await response.json().catch(() => null);
            throw new Error(errorData?.error || `HTTP ${response.status}: ${response.statusText}`);
        }

        const blob = await response.blob();
        return {
            blob,
            filename: normalizeMode === 'true' ? 'normalized_splices.zip' : 'normalized_audio.zip',
            type: 'normalize',
            targetLevel: document.getElementById('targetLevel').value,
            mode: normalizeMode === 'true' ? 'splices' : 'whole file'
        };
    }

    showProgress() {
        this.progress.style.display = 'block';
        this.progressBar.style.width = '100%';
    }

    hideProgress() {
        this.progress.style.display = 'none';
        this.progressBar.style.width = '0';
    }

    showResults(result, type) {
        this.currentBlob = result.blob;
        this.downloadButton.onclick = () => this.downloadFile(result.blob, result.filename);
        
        let infoHtml = '';
        if (type === 'splice') {
            infoHtml = `
                <strong>🎯 Splice Processing Complete</strong><br>
                Created ${result.count} splices of ${result.duration} seconds each<br>
                Reverse: ${result.reverse ? 'Yes' : 'No'}<br>
                File size: ${(result.blob.size / 1024).toFixed(1)} KB
            `;
        } else if (type === 'normalize') {
            infoHtml = `
                <strong>📊 Normalization Complete</strong><br>
                Target level: ${(result.targetLevel * 100).toFixed(0)}%<br>
                Mode: ${result.mode}<br>
                File size: ${(result.blob.size / 1024).toFixed(1)} KB
            `;
        }
        
        this.resultInfo.innerHTML = infoHtml;
        this.resultsSection.style.display = 'block';
    }

    hideResults() {
        this.resultsSection.style.display = 'none';
        this.currentBlob = null;
    }

    showError(message) {
        this.errorMessage.textContent = message;
        this.errorSection.style.display = 'block';
    }

    hideError() {
        this.errorSection.style.display = 'none';
    }

    downloadFile(blob, filename) {
        const url = window.URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = filename;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        window.URL.revokeObjectURL(url);
        
        this.updateStatus('Download started!', 'success');
    }

    downloadResults() {
        if (this.currentBlob) {
            const activeTab = document.querySelector('.tab-button.active').dataset.tab;
            const filename = activeTab === 'splice' ? 'audio_splices.zip' : 'normalized_audio.zip';
            this.downloadFile(this.currentBlob, filename);
        }
    }
}

// Initialize the application when the DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    new AudioProcessor();
});

// Add some CSS for status types
const statusStyle = document.createElement('style');
statusStyle.textContent = `
    .status.success {
        background: rgba(16, 185, 129, 0.8) !important;
    }
    
    .status.error {
        background: rgba(239, 68, 68, 0.8) !important;
    }
    
    .status.processing {
        background: rgba(245, 158, 11, 0.8) !important;
        animation: pulse 1.5s ease-in-out infinite;
    }
`;
document.head.appendChild(statusStyle);