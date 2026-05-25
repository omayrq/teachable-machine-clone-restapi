# Teachable Machine Clone — Rust Axum Backend + Streamlit Frontend + Gemini Vision

A hands-on full-stack AI image-classification project that recreates the basic workflow of Google Teachable Machine.

This project allows users to:

- Create and rename image classes
- Upload training images for each class
- Train or prepare a session
- Upload a test image
- Predict the image class using Gemini Vision
- Display confidence scores in a bar chart

---

## 1. Project Overview

This project is built as a practical learning project for full-stack AI development.

The frontend is built with **Streamlit**. The backend is built with **Rust Axum**. The real image prediction is handled by **Google Gemini Vision API**.

Example workflow:

```text
1. Rename Class 1 to cat
2. Rename Class 2 to dog
3. Upload cat images
4. Upload dog images
5. Click Train Model
6. Upload a test image
7. Click Predict
8. View confidence chart
```

---

## 2. Technology Stack

| Layer | Technology | Why we use it |
|---|---|---|
| Frontend | Streamlit | Quickly builds a browser-based UI using Python |
| Backend | Rust + Axum | Creates a fast and safe async API server |
| API Calls | REST API | Connects frontend and backend |
| Image Upload | Multipart Form Data | Sends images from frontend to backend |
| AI Model | Gemini Vision API | Performs real image classification |
| State Storage | In-memory HashMap | Temporarily stores uploaded images by session |
| Charting | Altair | Shows prediction confidence visually |
| Config | Environment Variable | Stores Gemini API key safely |

---

## 3. Final Project Structure

```text
teachable-machine-clone/
├── backend/
│   ├── Cargo.toml
│   ├── Cargo.lock
│   ├── src/
│   │   └── main.rs
│   └── target/
│
├── frontend/
│   ├── app.py
│   ├── requirements.txt
│   └── venv/
│
└── README.md
```

### Why this structure matters

- `backend/` contains the Rust API server.
- `frontend/` contains the Streamlit web app.
- `backend/src/main.rs` is the Rust application entry file.
- `backend/Cargo.toml` manages Rust dependencies.
- `frontend/requirements.txt` manages Python dependencies.
- `README.md` documents how to rebuild and run the project again.

---

## 4. Prerequisites

Install these before starting:

### Rust

Check installation:

```bash
rustc --version
cargo --version
```

Rust is required because the backend is written in Rust.

### Python

Check installation:

```bash
python --version
```

Python is required because the frontend is written in Streamlit.

### VS Code

VS Code is recommended for editing files:

```bash
code .
```

### Gemini API Key

Create a Gemini API key from Google AI Studio.

Set it temporarily in Git Bash:

```bash
export GEMINI_API_KEY="PASTE_YOUR_GEMINI_API_KEY_HERE"
```

Or save it permanently in Windows:

```bash
setx GEMINI_API_KEY "PASTE_YOUR_GEMINI_API_KEY_HERE"
```

After using `setx`, close VS Code and all terminals, then open them again.

Important:

```text
Never commit your real API key to GitHub.
```

---

## 5. Create Project Folder

```bash
cd /e/bootcamp_smit
mkdir teachable-machine-clone
cd teachable-machine-clone
mkdir backend frontend
```

Why this step is important:

- Keeps backend and frontend separated
- Makes the project clean and GitHub-ready
- Helps you practice professional project structure

---

## 6. Backend Setup

Go to backend folder:

```bash
cd /e/bootcamp_smit/teachable-machine-clone/backend
cargo init
```

This creates:

```text
backend/
├── Cargo.toml
└── src/
    └── main.rs
```

Why this step is important:

`cargo init` initializes a Rust application and creates the required Cargo project files.

---

## 7. Backend Cargo.toml

Open:

```bash
code Cargo.toml
```

Replace everything with:

```toml
[package]
name = "backend"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = { version = "0.7", features = ["multipart"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tower-http = { version = "0.6", features = ["cors"] }
base64 = "0.22"
reqwest = { version = "0.12", features = ["json"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }
```

### Why these dependencies are used

| Dependency | Usage |
|---|---|
| axum | Creates backend API routes |
| multipart | Allows image upload |
| tokio | Runs async Rust server |
| serde | Converts Rust structs to/from JSON |
| serde_json | Builds and parses JSON payloads |
| tower-http | Enables CORS |
| base64 | Converts image bytes into base64 for Gemini |
| reqwest | Sends HTTP request to Gemini API |
| tracing | Logging |
| tracing-subscriber | Displays logs in terminal |

---

## 8. Backend main.rs

Open:

```bash
code src/main.rs
```

Paste your final backend code that includes these endpoints:

```text
/api/ping
/api/upload
/api/train
/api/predict
/api/rename_class
```

Main backend responsibilities:

1. Start Axum server on `127.0.0.1:8000`
2. Accept image uploads from Streamlit
3. Store uploaded images temporarily in memory
4. Validate training session
5. Rename classes
6. Send test image to Gemini Vision API
7. Return prediction result to frontend

### Important backend concepts

#### AppState

Stores application data:

```text
session_id -> class_name -> images
```

This allows multiple browser sessions to upload their own images.

#### Multipart

Used because images are uploaded as files.

#### CORS

Needed because frontend runs on:

```text
localhost:8501
```

and backend runs on:

```text
127.0.0.1:8000
```

Without CORS, browser requests may be blocked.

#### Gemini API

The backend sends the test image to Gemini with a prompt asking it to classify the image into one of the class names.

---

## 9. Run Backend

From backend folder:

```bash
cd /e/bootcamp_smit/teachable-machine-clone/backend
export GEMINI_API_KEY="YOUR_GEMINI_API_KEY_HERE"
cargo run --release
```

Expected output:

```text
DEBUG backend: listening on 127.0.0.1:8000
```

Why this step is important:

This starts the API server. The frontend cannot work unless the backend is running.

---

## 10. Test Backend Health

Open a second terminal and run:

```bash
curl http://127.0.0.1:8000/api/ping
```

Expected output:

```text
pong
```

Why this step is important:

This confirms the backend is reachable before connecting the frontend.

---

## 11. Frontend Setup

Go to frontend folder:

```bash
cd /e/bootcamp_smit/teachable-machine-clone/frontend
```

Create virtual environment:

```bash
python -m venv venv
```

Activate it:

```bash
source venv/Scripts/activate
```

Why virtual environment is important:

It keeps this project's Python packages separate from other projects.

---

## 12. Frontend requirements.txt

Create:

```bash
code requirements.txt
```

Paste:

```txt
streamlit
requests
Pillow
altair
pandas
```

Install dependencies:

```bash
pip install -r requirements.txt
```

### Why these packages are used

| Package | Usage |
|---|---|
| streamlit | Creates browser UI |
| requests | Calls backend APIs |
| Pillow | Handles image display |
| altair | Creates confidence chart |
| pandas | Converts JSON prediction into chart data |

---

## 13. Frontend app.py

Create:

```bash
code app.py
```

Your Streamlit app should include:

- Session ID generation
- Class list state
- Image upload UI
- Rename class feature
- Train button
- Preview image upload
- Predict button
- Altair confidence chart

The frontend calls backend API:

```python
API_URL = "http://127.0.0.1:8000/api"
```

Why this is important:

This connects Streamlit UI to the Rust backend.

---

## 14. Run Frontend

Keep backend running in one terminal.

Open another terminal:

```bash
cd /e/bootcamp_smit/teachable-machine-clone/frontend
source venv/Scripts/activate
streamlit run app.py
```

Expected output:

```text
Local URL: http://localhost:8501
```

Open browser:

```text
http://localhost:8501
```

---

## 15. Full Real-Time Testing Workflow

1. Start backend.
2. Start frontend.
3. Open browser at `localhost:8501`.
4. Rename `Class 1` to `cat`.
5. Rename `Class 2` to `dog`.
6. Upload several cat images.
7. Click **Upload Files to cat**.
8. Upload several dog images.
9. Click **Upload Files to dog**.
10. Click **Train Model**.
11. Upload a test image.
12. Click **Predict**.
13. Check the output chart.

Expected behavior:

- If test image is dog, Gemini should classify it as dog.
- If test image is cat, Gemini should classify it as cat.

---

## 16. API Endpoints

| Endpoint | Method | Purpose |
|---|---|---|
| `/api/ping` | GET | Health check |
| `/api/upload` | POST | Upload image for a class |
| `/api/train` | POST | Validate session and classes |
| `/api/predict` | POST | Predict test image using Gemini |
| `/api/rename_class` | POST | Rename class |

---

## 17. Common Errors and Fixes

### Error: `cargo: command not found`

Rust is not installed or PATH is not updated.

Fix:

```bash
rustc --version
cargo --version
```

Restart terminal after installing Rust.

---

### Error: `Hello, world!`

You are still running default Rust code.

Fix:

```bash
code src/main.rs
```

Replace it with project backend code.

---

### Error: `cannot find crate axum/tokio`

Dependencies are missing from `Cargo.toml`.

Fix:

```bash
code Cargo.toml
```

Add required dependencies.

---

### Error: `no targets specified in the manifest`

Your `main.rs` is in the wrong location.

Correct location:

```text
backend/src/main.rs
```

Wrong location:

```text
backend/src/api/main.rs
```

Fix:

```bash
mv src/api/main.rs src/main.rs
```

---

### Error: `virtual manifest specifies a dependencies section`

Your `Cargo.toml` is missing `[package]`.

Fix:

```toml
[package]
name = "backend"
version = "0.1.0"
edition = "2021"
```

---

### Error: `Access is denied`

Old backend process is still running.

Fix:

```bash
taskkill //F //IM backend.exe
cargo run --release
```

---

### Error: frontend connection error

Backend is not running.

Fix:

```bash
curl http://127.0.0.1:8000/api/ping
```

Expected:

```text
pong
```

---

### Error: prediction always shows first class

This happens when mock prediction logic is still being used.

Fix:

- Add Gemini prediction code
- Set `GEMINI_API_KEY`
- Restart backend
- Test again

---

### Error: `GEMINI_API_KEY is missing`

Fix in Git Bash:

```bash
export GEMINI_API_KEY="YOUR_KEY"
cargo run --release
```

Or permanent Windows setup:

```bash
setx GEMINI_API_KEY "YOUR_KEY"
```

Then restart VS Code.

---

## 18. Why the First Version Predicted Wrong

The first prediction version was mock logic:

```rust
let conf = if i == 0 { 0.85 } else { 0.15 / (num_classes - 1) as f32 };
```

This always gave the first class 85% confidence.

So if your first class was `cat`, every image looked like `cat` in the output.

The fixed version sends the uploaded test image to Gemini Vision API for real classification.

---

## 19. Security Notes

Never commit API keys to GitHub.

Create `.gitignore`:

```gitignore
# Rust
target/

# Python
venv/
__pycache__/
*.pyc

# Secrets
.env
```

If you accidentally expose an API key, revoke it and create a new key.

---

## 20. GitHub Upload Commands

From project root:

```bash
git init
git add .
git commit -m "Initial commit: Teachable Machine Clone with Rust Streamlit and Gemini"
```

Connect GitHub repo:

```bash
git remote add origin https://github.com/YOUR_USERNAME/teachable-machine-clone.git
git branch -M main
git push -u origin main
```

---

## 21. Future Improvements

You can improve this project by adding:

1. SQLite or PostgreSQL instead of in-memory HashMap
2. Dockerfile for backend
3. Dockerfile for frontend
4. Docker Compose
5. Few-shot Gemini prompt using uploaded training images
6. Authentication
7. Cloud deployment
8. Better UI design
9. Dataset versioning
10. Prediction history
11. Logging dashboard
12. Unit tests

---

## 22. Final Hands-On Practice Checklist

Use this checklist whenever you rebuild this project:

```text
1. Create project folder
2. Create backend and frontend folders
3. Run cargo init inside backend
4. Update Cargo.toml
5. Add backend main.rs
6. Set Gemini API key
7. Run backend
8. Test /api/ping
9. Create Python virtual environment
10. Add requirements.txt
11. Install frontend dependencies
12. Add app.py
13. Run Streamlit frontend
14. Rename classes
15. Upload training images
16. Train model
17. Upload test image
18. Predict result
19. Debug if needed
20. Push project to GitHub
```

---

## 23. What You Learned

By completing this project, you practiced:

- Rust backend development
- Axum REST API routing
- Multipart image upload
- Streamlit UI development
- Frontend-backend integration
- Gemini Vision API integration
- Environment variable handling
- Debugging real-world project errors
- GitHub-ready documentation

This is a strong project for an AI engineering, data engineering, or full-stack AI portfolio.
