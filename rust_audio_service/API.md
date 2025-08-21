# Rust Audio Service API Documentation

## Overview

The Rust Audio Service is a modular audio processing API that provides audio splicing, effects, and analysis capabilities. The service supports both legacy endpoints (for backward compatibility) and modern versioned APIs.

**Base URL:** `http://127.0.0.1:8081`

## API Endpoints

### Health Check

**GET** `/api/v1/health`

Returns service health status and uptime information.

**Response:**
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime_seconds": 3600
}
```

**Status Codes:**
- `200 OK` - Service is healthy

---

### Audio Splicing (Legacy)

**POST** `/process`

Legacy endpoint for backward compatibility with existing Go CLI. Accepts multipart form data and returns a ZIP file containing audio splices.

**Content-Type:** `multipart/form-data`

**Form Fields:**
- `file` - Audio file (WAV format)
- `spliceDuration` - Duration of each splice in seconds (float)
- `spliceCount` - Number of splices to create (integer)
- `reverse` - Whether to reverse audio samples (boolean, "true"/"false")

**Response:**
- Content-Type: `application/zip`
- Body: ZIP file containing splice files named `splice_0.wav`, `splice_1.wav`, etc.

**Status Codes:**
- `200 OK` - Processing successful, ZIP file returned
- `400 Bad Request` - Invalid parameters or validation failed
- `500 Internal Server Error` - Processing failed

**Example using curl:**
```bash
curl -X POST http://127.0.0.1:8081/process \
  -F "file=@audio.wav" \
  -F "spliceDuration=2.0" \
  -F "spliceCount=5" \
  -F "reverse=false" \
  --output splices.zip
```

---

### Audio Splicing (Versioned)

**POST** `/api/v1/audio/splice/multipart`

Modern versioned endpoint with enhanced error handling and response format.

**Content-Type:** `multipart/form-data`

**Form Fields:** (Same as legacy endpoint)
- `file` - Audio file (WAV format)
- `spliceDuration` - Duration of each splice in seconds (float)
- `spliceDuration` - Number of splices to create (integer)
- `reverse` - Whether to reverse audio samples (boolean)

**Response:**
- Content-Type: `application/zip`
- Body: ZIP file containing splice files

**Error Response:**
```json
{
  "success": false,
  "result": null,
  "error": "Invalid splice duration: splice_duration must be positive"
}
```

**Status Codes:**
- `200 OK` - Processing successful
- `400 Bad Request` - Invalid parameters
- `500 Internal Server Error` - Processing failed

---

### Audio Splicing (JSON) - Not Yet Implemented

**POST** `/api/v1/audio/splice`

JSON-based endpoint for future file upload integration.

**Status:** Currently returns `501 Not Implemented`

---

## Processing Configuration

### Splice Configuration

```json
{
  "type": "splice",
  "duration": 2.0,
  "count": 5,
  "reverse": false
}
```

**Parameters:**
- `duration` (float) - Duration of each splice in seconds, must be > 0
- `count` (integer) - Number of splices to generate, must be >= 1
- `reverse` (boolean) - Whether to reverse the audio samples in each splice

---

## Processing Metadata

The service tracks processing performance and includes metadata in internal responses:

```json
{
  "processor_type": "splice",
  "input_duration": 120.5,
  "sample_rate": 44100,
  "channels": 2,
  "processing_time_ms": 1250
}
```

---

## Error Handling

### Error Types

- **InvalidDuration** - Splice duration is invalid (≤ 0)
- **InvalidSpliceCount** - Splice count is invalid (< 1)
- **ProcessingError** - General processing failure
- **IoError** - File I/O error
- **WavError** - WAV file format error

### Error Response Format (Versioned APIs)

```json
{
  "error": "Human-readable error message",
  "error_type": "ErrorTypeName",
  "timestamp": "2025-01-01T12:00:00Z"
}
```

---

## Usage Examples

### Basic Splicing (Legacy)

```bash
# Create 3 splices of 1.5 seconds each
curl -X POST http://127.0.0.1:8081/process \
  -F "file=@song.wav" \
  -F "spliceDuration=1.5" \
  -F "spliceCount=3" \
  -F "reverse=false" \
  --output output.zip
```

### Reversed Splicing

```bash
# Create 5 reversed splices of 2.0 seconds each
curl -X POST http://127.0.0.1:8081/process \
  -F "file=@song.wav" \
  -F "spliceDuration=2.0" \
  -F "spliceCount=5" \
  -F "reverse=true" \
  --output reversed_splices.zip
```

### Health Check

```bash
curl http://127.0.0.1:8081/api/v1/health
```

---

## Rate Limits and Constraints

- **File Size**: No explicit limits (limited by system memory)
- **Processing Time**: Depends on file size and splice count
- **Concurrent Requests**: Limited by system resources
- **File Format**: Currently supports WAV files only
- **Temporary Files**: Automatically cleaned up after processing

---

## Development Notes

- All temporary files are stored in `/tmp/` directory
- Audio processing uses 16-bit signed integer samples
- Random splice selection uses uniform distribution
- ZIP files use no compression (stored method) for faster processing