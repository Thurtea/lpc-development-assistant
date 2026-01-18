#!/usr/bin/env python3
"""
LPC MUD Development Assistant - Python GUI
A native GUI for querying Ollama models with MUD development context.
"""

import sys
import json
import requests
from pathlib import Path
from datetime import datetime
from PyQt6.QtWidgets import (
    QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout,
    QPushButton, QTextEdit, QComboBox, QLabel, QMessageBox, QSplitter
)
from PyQt6.QtCore import Qt, QThread, pyqtSignal
from PyQt6.QtGui import QFont, QTextOption, QPalette, QColor
from pygments import highlight
from pygments.lexers import CLexer
from pygments.formatters import HtmlFormatter


class OllamaWorker(QThread):
    """Background thread for Ollama API calls."""
    finished = pyqtSignal(str)
    error = pyqtSignal(str)

    def __init__(self, model, prompt, context_text):
        super().__init__()
        self.model = model
        self.prompt = prompt
        self.context_text = context_text

    def run(self):
        try:
            # Combine context + prompt
            full_prompt = self.context_text + "\n\n---\n\nQUESTION:\n" + self.prompt
            full_prompt += "\n\nProvide complete, production-ready code with detailed comments."

            # Call Ollama HTTP API
            response = requests.post(
                "http://localhost:11434/api/generate",
                json={
                    "model": self.model,
                    "prompt": full_prompt,
                    "stream": False,
                    "options": {
                        "temperature": 0.3,
                        "top_p": 0.9,
                        "num_predict": 4096
                    }
                },
                timeout=300
            )

            if response.status_code == 200:
                data = response.json()
                self.finished.emit(data.get("response", "No response"))
            else:
                self.error.emit(f"Ollama error: {response.status_code}")

        except requests.exceptions.ConnectionError:
            self.error.emit("Cannot connect to Ollama. Is it running? (ollama serve)")
        except Exception as e:
            self.error.emit(f"Error: {str(e)}")


class LPCDevGUI(QMainWindow):
    """Main GUI window."""

    def __init__(self):
        super().__init__()
        self.workspace_root = Path("E:/Work/AMLP")
        self.templates_path = self.workspace_root / "lpc-dev-assistant" / "templates"
        self.gen_path = self.workspace_root / "lpc-dev-assistant" / "gen"
        self.gen_path.mkdir(exist_ok=True)
        
        self.worker = None
        self.current_response = ""
        
        self.init_ui()
        self.load_models()
        self.apply_dark_theme()

    def init_ui(self):
        """Initialize the user interface."""
        self.setWindowTitle("LPC MUD Development Assistant")
        self.setGeometry(100, 100, 1400, 900)

        # Central widget
        central_widget = QWidget()
        self.setCentralWidget(central_widget)
        main_layout = QVBoxLayout(central_widget)

        # Title
        title = QLabel("🎮 LPC MUD Development Assistant")
        title_font = QFont("Segoe UI", 18, QFont.Weight.Bold)
        title.setFont(title_font)
        main_layout.addWidget(title)

        # Controls row
        controls_layout = QHBoxLayout()

        # Model selection
        self.model_combo = QComboBox()
        self.model_combo.setMinimumWidth(250)
        controls_layout.addWidget(QLabel("Model:"))
        controls_layout.addWidget(self.model_combo)

        # Context selection
        self.context_combo = QComboBox()
        self.context_combo.addItems([
            "Driver Development",
            "Efuns Implementation", 
            "MudLib/LPC Code",
            "Reference Libraries"
        ])
        self.context_combo.setMinimumWidth(200)
        controls_layout.addWidget(QLabel("Context:"))
        controls_layout.addWidget(self.context_combo)

        controls_layout.addStretch()

        # Buttons
        self.ask_btn = QPushButton("🚀 Ask Ollama")
        self.ask_btn.clicked.connect(self.ask_ollama)
        self.ask_btn.setMinimumHeight(35)
        controls_layout.addWidget(self.ask_btn)

        self.save_btn = QPushButton("💾 Save")
        self.save_btn.clicked.connect(self.save_response)
        self.save_btn.setEnabled(False)
        self.save_btn.setMinimumHeight(35)
        controls_layout.addWidget(self.save_btn)

        self.search_btn = QPushButton("🔍 Search Refs")
        self.search_btn.clicked.connect(self.search_references)
        self.search_btn.setMinimumHeight(35)
        controls_layout.addWidget(self.search_btn)

        main_layout.addLayout(controls_layout)

        # Status label
        self.status_label = QLabel("")
        self.status_label.setStyleSheet("color: #4CAF50; font-size: 12px;")
        main_layout.addWidget(self.status_label)

        # Splitter for question and response
        splitter = QSplitter(Qt.Orientation.Vertical)

        # Question input
        question_widget = QWidget()
        question_layout = QVBoxLayout(question_widget)
        question_layout.setContentsMargins(0, 0, 0, 0)
        
        question_label = QLabel("📝 Your Question:")
        question_label.setStyleSheet("font-size: 14px; font-weight: bold;")
        question_layout.addWidget(question_label)

        self.question_input = QTextEdit()
        self.question_input.setPlaceholderText(
            "Ask about LPC driver implementation, mudlib features, or C programming...\n\n"
            "Examples:\n"
            "- Write the complete lexer.c for LPC tokens\n"
            "- Implement the VM bytecode interpreter\n"
            "- Create a combat system for the mudlib"
        )
        self.question_input.setFont(QFont("Consolas", 11))
        self.question_input.setMinimumHeight(120)
        question_layout.addWidget(self.question_input)

        splitter.addWidget(question_widget)

        # Response display
        response_widget = QWidget()
        response_layout = QVBoxLayout(response_widget)
        response_layout.setContentsMargins(0, 0, 0, 0)

        response_label = QLabel("💬 Response:")
        response_label.setStyleSheet("font-size: 14px; font-weight: bold;")
        response_layout.addWidget(response_label)

        self.response_display = QTextEdit()
        self.response_display.setReadOnly(True)
        self.response_display.setFont(QFont("Consolas", 10))
        self.response_display.setPlaceholderText(
            "Response will appear here...\n\n"
            "💡 Tip: Select a model, choose context, and ask a question!"
        )
        self.response_display.setLineWrapMode(QTextEdit.LineWrapMode.NoWrap)
        response_layout.addWidget(self.response_display)

        splitter.addWidget(response_widget)

        # References display
        refs_widget = QWidget()
        refs_layout = QVBoxLayout(refs_widget)
        refs_layout.setContentsMargins(0, 0, 0, 0)

        refs_label = QLabel("📚 References:")
        refs_label.setStyleSheet("font-size: 14px; font-weight: bold;")
        refs_layout.addWidget(refs_label)

        self.refs_display = QTextEdit()
        self.refs_display.setReadOnly(True)
        self.refs_display.setFont(QFont("Consolas", 9))
        self.refs_display.setPlaceholderText("No references searched yet")
        self.refs_display.setMaximumHeight(150)
        refs_layout.addWidget(self.refs_display)

        splitter.addWidget(refs_widget)

        # Set splitter sizes
        splitter.setSizes([150, 500, 150])
        main_layout.addWidget(splitter)

    def apply_dark_theme(self):
        """Apply dark theme to the application."""
        dark_palette = QPalette()
        dark_palette.setColor(QPalette.ColorRole.Window, QColor(30, 30, 35))
        dark_palette.setColor(QPalette.ColorRole.WindowText, QColor(220, 220, 220))
        dark_palette.setColor(QPalette.ColorRole.Base, QColor(25, 25, 30))
        dark_palette.setColor(QPalette.ColorRole.AlternateBase, QColor(35, 35, 40))
        dark_palette.setColor(QPalette.ColorRole.ToolTipBase, QColor(220, 220, 220))
        dark_palette.setColor(QPalette.ColorRole.ToolTipText, QColor(220, 220, 220))
        dark_palette.setColor(QPalette.ColorRole.Text, QColor(220, 220, 220))
        dark_palette.setColor(QPalette.ColorRole.Button, QColor(45, 45, 50))
        dark_palette.setColor(QPalette.ColorRole.ButtonText, QColor(220, 220, 220))
        dark_palette.setColor(QPalette.ColorRole.BrightText, QColor(255, 255, 255))
        dark_palette.setColor(QPalette.ColorRole.Link, QColor(42, 130, 218))
        dark_palette.setColor(QPalette.ColorRole.Highlight, QColor(42, 130, 218))
        dark_palette.setColor(QPalette.ColorRole.HighlightedText, QColor(255, 255, 255))

        self.setPalette(dark_palette)
        self.setStyleSheet("""
            QPushButton {
                background-color: #2d5a8c;
                border: none;
                padding: 8px 16px;
                border-radius: 4px;
                color: white;
                font-weight: bold;
            }
            QPushButton:hover {
                background-color: #3a6fa5;
            }
            QPushButton:pressed {
                background-color: #1e4266;
            }
            QPushButton:disabled {
                background-color: #3a3a3f;
                color: #888;
            }
            QComboBox {
                padding: 5px;
                background-color: #2d2d32;
                border: 1px solid #4a4a4f;
                border-radius: 3px;
            }
            QTextEdit {
                background-color: #1e1e23;
                border: 1px solid #3a3a3f;
                border-radius: 4px;
                padding: 8px;
            }
        """)

    def load_models(self):
        """Load available Ollama models."""
        try:
            response = requests.get("http://localhost:11434/api/tags", timeout=5)
            if response.status_code == 200:
                data = response.json()
                models = [m["name"] for m in data.get("models", [])]
                if models:
                    self.model_combo.addItems(models)
                    return
        except:
            pass

        # Fallback models
        self.model_combo.addItems(["qwen2.5-coder:7b", "qwen2.5:7b-instruct", "llama2:latest"])

    def load_context(self, context_name):
        """Load context template from file."""
        context_files = {
            "Driver Development": "driver_context.txt",
            "Efuns Implementation": "efuns_context.txt",
            "MudLib/LPC Code": "mudlib_context.txt",
            "Reference Libraries": "reference_sources.txt"
        }

        filename = context_files.get(context_name, "")
        if not filename:
            return ""

        context_path = self.templates_path / filename
        try:
            return context_path.read_text(encoding='utf-8')
        except Exception as e:
            self.show_status(f"Could not load context: {e}", error=True)
            return ""

    def ask_ollama(self):
        """Send question to Ollama."""
        question = self.question_input.toPlainText().strip()
        if not question:
            self.show_status("Please enter a question", error=True)
            return

        model = self.model_combo.currentText()
        if not model:
            self.show_status("Please select a model", error=True)
            return

        context_name = self.context_combo.currentText()
        context_text = self.load_context(context_name)

        # Disable UI during generation
        self.ask_btn.setEnabled(False)
        self.ask_btn.setText("⏳ Generating...")
        self.response_display.setPlainText("Generating response from Ollama...\nThis may take 30-60 seconds.")
        self.show_status("Generating response...")

        # Start worker thread
        self.worker = OllamaWorker(model, question, context_text)
        self.worker.finished.connect(self.on_response_received)
        self.worker.error.connect(self.on_error)
        self.worker.start()

    def on_response_received(self, response):
        """Handle received response."""
        self.current_response = response
        self.response_display.setPlainText(response)
        self.save_btn.setEnabled(True)
        self.ask_btn.setEnabled(True)
        self.ask_btn.setText("🚀 Ask Ollama")
        self.show_status("✅ Response received!")

    def on_error(self, error_msg):
        """Handle errors."""
        self.response_display.setPlainText(f"Error: {error_msg}")
        self.ask_btn.setEnabled(True)
        self.ask_btn.setText("🚀 Ask Ollama")
        self.show_status(error_msg, error=True)

    def save_response(self):
        """Save response to file."""
        if not self.current_response:
            return

        context_names = {
            "Driver Development": "driver",
            "Efuns Implementation": "efuns",
            "MudLib/LPC Code": "mudlib",
            "Reference Libraries": "reference"
        }

        context = context_names.get(self.context_combo.currentText(), "output")
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        filename = f"{context}_{timestamp}.c"
        filepath = self.gen_path / filename

        try:
            filepath.write_text(self.current_response, encoding='utf-8')
            self.show_status(f"✅ Saved to: {filename}")
        except Exception as e:
            self.show_status(f"Save failed: {e}", error=True)

    def search_references(self):
        """Search MUD reference files."""
        refs_path = self.workspace_root / "mud-references" / "extracted"
        if not refs_path.exists():
            self.refs_display.setPlainText("References not extracted yet.\nRun extract in the mud-references folder.")
            return

        self.show_status("Searching references...")
        results = []

        for ext in ['.c', '.h', '.lpc']:
            for file in refs_path.rglob(f"*{ext}"):
                results.append(str(file))
                if len(results) >= 100:
                    break
            if len(results) >= 100:
                break

        if results:
            output = "Found references:\n\n"
            for i, path in enumerate(results[:50], 1):
                output += f"{i}. {path}\n"
            if len(results) > 50:
                output += f"\n... and {len(results) - 50} more files"
            self.refs_display.setPlainText(output)
            self.show_status(f"Found {len(results)} reference files")
        else:
            self.refs_display.setPlainText("No reference files found")
            self.show_status("No references found")

    def show_status(self, message, error=False):
        """Show status message."""
        color = "#f44336" if error else "#4CAF50"
        self.status_label.setStyleSheet(f"color: {color}; font-size: 12px;")
        self.status_label.setText(message)


def main():
    app = QApplication(sys.argv)
    app.setStyle("Fusion")
    window = LPCDevGUI()
    window.show()
    sys.exit(app.exec())


if __name__ == "__main__":
    main()
