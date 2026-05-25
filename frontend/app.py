import streamlit as st
import requests
import uuid
import time
import io
from PIL import Image

# Config
st.set_page_config(page_title="Teachable Machine Clone", layout="wide")
API_URL = "http://127.0.0.1:8000/api"

# Session State Initialization
if "session_id" not in st.session_state:
    st.session_state["session_id"] = str(uuid.uuid4())
if "classes" not in st.session_state:
    st.session_state["classes"] = ["Class 1", "Class 2"]
if "trained" not in st.session_state:
    st.session_state["trained"] = False
if "training_in_progress" not in st.session_state:
    st.session_state["training_in_progress"] = False
if "prediction_result" not in st.session_state:
    st.session_state["prediction_result"] = None

st.title("Teachable Machine Clone")

# Layout: 3 columns
col_classes, col_training, col_preview = st.columns([2, 1, 1.5])

# --- Helper Functions ---
def upload_image_to_backend(class_name, image_bytes):
    try:
        files = {'file': image_bytes}
        data = {'session_id': st.session_state["session_id"], 'class_name': class_name}
        r = requests.post(f"{API_URL}/upload", files=files, data=data)
        if r.status_code == 200:
            return True
        else:
            st.error(f"Upload failed: {r.text}")
            return False
    except Exception as e:
        st.error(f"Connection error: {e}")
        return False

def rename_class_in_backend(old_name, new_name):
    try:
        data = {
            'session_id': st.session_state["session_id"],
            'old_class_name': old_name,
            'new_class_name': new_name
        }
        r = requests.post(f"{API_URL}/rename_class", json=data)
        if r.status_code == 200:
            return True
        else:
            st.error(f"Rename failed: {r.text}")
            return False
    except Exception as e:
        st.error(f"Connection error: {e}")
        return False

def save_class_name(index, old_name):
    new_name = st.session_state.get(f"input_{index}")
    if new_name and new_name != old_name:
        # Optimistically update or check backend
        if rename_class_in_backend(old_name, new_name):
            st.session_state["classes"][index] = new_name
        else:
            st.error("Failed to rename class in backend.")
    st.session_state[f"editing_{index}"] = False

# --- Column 1: Classes ---
with col_classes:
    st.subheader("Classes")
    for i, cls in enumerate(st.session_state["classes"]):
        with st.expander(cls, expanded=True):
            # Edit Class Name Logic
            col_name, col_edit = st.columns([0.8, 0.2])
            with col_name:
                if st.session_state.get(f"editing_{i}", False):
                    st.text_input(
                        "New Name", 
                        value=cls, 
                        key=f"input_{i}", 
                        label_visibility="collapsed",
                        on_change=save_class_name,
                        args=(i, cls)
                    )
                else:
                    st.write(f"### {cls}")
            with col_edit:
                if not st.session_state.get(f"editing_{i}", False):
                    if st.button("✏️", key=f"edit_{i}"):
                        st.session_state[f"editing_{i}"] = True
                        st.rerun()

            # To simulate tabs for Webcam vs File Upload
            tab1, tab2 = st.tabs(["Upload", "Webcam"])
            with tab1:
                uploaded_files = st.file_uploader(f"Add Images for {cls}", accept_multiple_files=True, key=f"file_{i}")
                if st.button(f"Upload Files to {cls}", key=f"btn_upload_{i}"):
                    if uploaded_files:
                        success = 0
                        with st.spinner("Uploading..."):
                            for file in uploaded_files:
                                if upload_image_to_backend(cls, file.getvalue()):
                                    success += 1
                        st.success(f"Uploaded {success} images to {cls}")
            with tab2:
                # Streamlit's camera_input takes a picture when clicked. 
                # (For true real-time, Streamlit WebRTC is needed, but camera_input works for cloning the data collection step)
                cam_img = st.camera_input("Take a picture", key=f"cam_{i}")
                if cam_img is not None:
                    if st.button("Add to " + cls, key=f"btn_cam_{i}"):
                        if upload_image_to_backend(cls, cam_img.getvalue()):
                            st.success("Image added!")

    if st.button("+ Add a class"):
        st.session_state["classes"].append(f"Class {len(st.session_state['classes']) + 1}")
        st.rerun()

# --- Column 2: Training ---
with col_training:
    st.subheader("Training")
    st.write("Train your model after uploading samples.")
    if st.button("Train Model", use_container_width=True, type="primary"):
        st.session_state["training_in_progress"] = True
        
    if st.session_state.get("training_in_progress"):
        with st.spinner("Preparing model / Training..."):
            try:
                data = {'session_id': st.session_state["session_id"]}
                r = requests.post(f"{API_URL}/train", json=data)
                if r.status_code == 200:
                    st.session_state["trained"] = True
                    st.success("Training Complete!")
                else:
                    st.error(f"Training failed: {r.text}")
            except Exception as e:
                st.error(f"Connection error: {e}")
        st.session_state["training_in_progress"] = False

# --- Column 3: Preview ---
with col_preview:
    st.subheader("Preview")
    st.write("Test your trained model here.")
    
    if not st.session_state["trained"]:
        st.info("You must train a model on the left before you can preview it here.")
    else:
        test_tab1, test_tab2 = st.tabs(["File Upload", "Webcam"])
        test_img_bytes = None
        
        def clear_prediction():
            st.session_state["prediction_result"] = None
            
        with test_tab1:
            test_file = st.file_uploader("Upload Test Image", key="test_file", on_change=clear_prediction)
            if test_file:
                test_img_bytes = test_file.getvalue()
                st.image(test_file, width=300)
                
        with test_tab2:
            test_cam = st.camera_input("Webcam Test", key="test_cam", on_change=clear_prediction)
            if test_cam:
                test_img_bytes = test_cam.getvalue()

        if test_img_bytes and st.button("Predict", use_container_width=True):
            with st.spinner("Predicting..."):
                try:
                    files = {'file': test_img_bytes}
                    data = {'session_id': st.session_state["session_id"]}
                    r = requests.post(f"{API_URL}/predict", files=files, data=data)
                    if r.status_code == 200:
                        res = r.json()
                        st.session_state["prediction_result"] = res
                    else:
                        st.error(f"Prediction failed: {r.text}")
                except Exception as e:
                    st.error(f"Connection error: {e}")

        if st.session_state["prediction_result"]:
            res = st.session_state["prediction_result"]
            st.write("### Output")
            
            if "predictions" in res:
                import pandas as pd
                import altair as alt
                
                df = pd.DataFrame(res["predictions"])
                
                # Create Altair bar chart with different colors for each class
                chart = alt.Chart(df).mark_bar().encode(
                    x=alt.X('confidence:Q', scale=alt.Scale(domain=[0, 1]), axis=alt.Axis(format='%', title='Confidence')),
                    y=alt.Y('class_name:N', sort='-x', axis=alt.Axis(title='')),
                    color=alt.Color('class_name:N', legend=None),
                    tooltip=['class_name', alt.Tooltip('confidence:Q', format='.1%')]
                ).properties(height=150)
                
                st.altair_chart(chart, use_container_width=True)
            else:
                st.error("Unexpected response format from backend.")
