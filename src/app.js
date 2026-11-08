/**
 * JARVIS VOICE CHAT - 100% LOCAL DESKTOP APP
 * Tauri 2.0 Frontend Client Logic
 */

(function () {
  'use strict';

  // --- Tauri IPC Bridge Helper ---
  async function invoke(cmd, args = {}) {
    if (window.__TAURI__ && window.__TAURI__.core && typeof window.__TAURI__.core.invoke === 'function') {
      return await window.__TAURI__.core.invoke(cmd, args);
    } else {
      console.warn(`[Tauri Mock] invoke("${cmd}") called without native backend.`, args);
      // Browser fallback mock for development UI preview
      return mockBackendResponse(cmd, args);
    }
  }

  // --- Mock responses for browser development preview ---
  async function mockBackendResponse(cmd, args) {
    if (cmd === 'get_status') {
      return { ollama_running: false, whisper_model_present: true, is_recording: false, is_speaking: false };
    }
    if (cmd === 'get_ollama_models') {
      throw new Error("Ollama is not running. Run 'ollama serve'");
    }
    if (cmd === 'get_microphones') {
      return ["Default Microphone (System Audio)"];
    }
    if (cmd === 'get_tts_voices') {
      return [{ id: 'default', name: 'Young Female Voice (en-us+f5)', language: 'en-us' }];
    }
    if (cmd === 'get_gptsovits_status') {
      return false; // mock: server not running in browser preview
    }
    if (cmd === 'get_chat_history') {
      return [];
    }
    if (cmd === 'start_recording' || cmd === 'cancel_recording' || cmd === 'save_chat_history' || cmd === 'stop_speaking') {
      return null;
    }
    if (cmd === 'stop_recording_and_transcribe') {
      return "Hello Rina-chan, how are you today?";
    }
    if (cmd === 'send_ollama_chat') {
      return "Rina-chan Board: Egao (Smile)! I'm doing great, thank you for talking with me! ♪";
    }
    if (cmd === 'export_chat_history') {
      return "C:/Users/User/Downloads/Rina_Transcript_Diary.txt";
    }
    return null;
  }

  const MODEL_PRESETS = [
    { id: 'smollm2:360m', name: 'smollm2:360m (0.36B - ~230MB, Ultra-Light)', group: '🌸 Ultra-Light (0.3B ~ 0.5B)' },
    { id: 'qwen2.5:0.5b', name: 'qwen2.5:0.5b (0.5B - ~398MB, Default Fast)', group: '🌸 Ultra-Light (0.3B ~ 0.5B)' },
    { id: 'qwen2:0.5b', name: 'qwen2:0.5b (0.5B - ~350MB)', group: '🌸 Ultra-Light (0.3B ~ 0.5B)' },
    { id: 'llama3.2:1b', name: 'llama3.2:1b (1B - ~1.3GB, Meta)', group: '⚡ Compact (1B ~ 2B)' },
    { id: 'qwen2.5:1.5b', name: 'qwen2.5:1.5b (1.5B - ~986MB, High Quality)', group: '⚡ Compact (1B ~ 2B)' },
    { id: 'deepseek-r1:1.5b', name: 'deepseek-r1:1.5b (1.5B - ~1.1GB, Reasoning)', group: '⚡ Compact (1B ~ 2B)' },
    { id: 'smollm2:1.7b', name: 'smollm2:1.7b (1.7B - ~1.0GB)', group: '⚡ Compact (1B ~ 2B)' },
    { id: 'gemma2:2b', name: 'gemma2:2b (2B - ~1.6GB, Google)', group: '⚡ Compact (1B ~ 2B)' },
    { id: 'llama3.2:3b', name: 'llama3.2:3b (3B - ~2.0GB, Meta)', group: '💡 Balanced (3B ~ 4B)' },
    { id: 'qwen2.5:3b', name: 'qwen2.5:3b (3B - ~1.9GB)', group: '💡 Balanced (3B ~ 4B)' },
    { id: 'phi3.5:3.8b', name: 'phi3.5:3.8b (3.8B - ~2.2GB, Microsoft)', group: '💡 Balanced (3B ~ 4B)' },
    { id: 'phi4-mini:3.8b', name: 'phi4-mini:3.8b (3.8B - ~2.4GB)', group: '💡 Balanced (3B ~ 4B)' },
    { id: 'qwen2.5:7b', name: 'qwen2.5:7b (7B - ~4.7GB, Top Tier)', group: '🧠 Heavy / High-IQ (7B ~ 8B)' },
    { id: 'mistral:7b', name: 'mistral:7b (7B - ~4.1GB, Mistral AI)', group: '🧠 Heavy / High-IQ (7B ~ 8B)' },
    { id: 'deepseek-r1:7b', name: 'deepseek-r1:7b (7B - ~4.7GB, Reasoning)', group: '🧠 Heavy / High-IQ (7B ~ 8B)' },
    { id: 'llama3.1:8b', name: 'llama3.1:8b (8B - ~4.7GB, Meta Llama 3.1)', group: '🧠 Heavy / High-IQ (7B ~ 8B)' }
  ];

  // --- Application State ---
  const state = {
    appState: 'ready', // 'ready' | 'listening' | 'thinking' | 'speaking'
    chatHistory: [],
    models: MODEL_PRESETS.map(m => m.id),
    selectedModel: 'qwen2.5:0.5b',
    systemPrompt: "You are Rina Tennoji (Rina-chan), the cute, shy, and tech-savvy school idol from Nijigasaki High School! You express your feelings through your beloved digital Rina-chan Board. Occasionally include cute board expressions like 'Rina-chan Board: Egao!' or 'Rina-chan Board: Wakuwaku!'. Keep your answers sweet, friendly, helpful, and concise (1-2 sentences) so they sound adorable spoken out loud!",
    temperature: 0.70,
    microphones: [],
    selectedMicrophone: '',
    voices: [],
    selectedVoice: 'default',
    speechSpeed: 1.0,
    volume: 1.0,
    autoSpeak: true,
    useEdgeTts: false,
    whisperModelReady: false,
    spacebarDown: false,
    recordingStartTime: 0
  };

  // --- DOM Elements ---
  const DOM = {
    // Header & Status
    statusPill: document.getElementById('status-pill'),
    statusLabel: document.getElementById('status-label'),
    headerFace: document.getElementById('header-face'),
    heroFace: document.getElementById('hero-face'),
    heroCaption: document.getElementById('hero-caption'),
    btnTalkFace: document.getElementById('btn-talk-face'),
    globalAlert: document.getElementById('global-alert'),
    alertMessage: document.getElementById('alert-message'),
    alertActionBtn: document.getElementById('alert-action-btn'),
    alertDismissBtn: document.getElementById('alert-dismiss-btn'),

    // Chat Viewport
    messagesContainer: document.getElementById('messages-container'),
    heroWelcome: document.getElementById('hero-welcome'),

    // Bottom Voice Controls
    btnTalk: document.getElementById('btn-talk'),
    btnStopSpeech: document.getElementById('btn-stop-speech'),
    textChatForm: document.getElementById('text-chat-form'),
    chatInput: document.getElementById('chat-input'),
    btnSendText: document.getElementById('btn-send-text'),
    quickVolume: document.getElementById('quick-volume'),

    // Header Quick Buttons
    btnExport: document.getElementById('btn-export'),
    btnClear: document.getElementById('btn-clear'),
    btnSettingsOpen: document.getElementById('btn-settings-open'),
    btnSettingsClose: document.getElementById('btn-settings-close'),
    btnSaveSettings: document.getElementById('btn-save-settings'),

    // Settings Modal
    settingsModal: document.getElementById('settings-modal'),
    modalTabs: document.querySelectorAll('.tab-btn'),
    tabPanes: document.querySelectorAll('.tab-pane'),

    // Ollama Settings
    ollamaStatusBadge: document.getElementById('ollama-status-badge'),
    selectOllamaModel: document.getElementById('select-ollama-model'),
    btnRefreshModels: document.getElementById('btn-refresh-models'),
    inputPullModel: document.getElementById('input-pull-model'),
    btnPullModel: document.getElementById('btn-pull-model'),
    pullStatusMsg: document.getElementById('pull-status-msg'),
    sliderTemp: document.getElementById('slider-temp'),
    tempValDisplay: document.getElementById('temp-val-display'),
    textareaSysPrompt: document.getElementById('textarea-sysprompt'),
    btnResetSysPrompt: document.getElementById('btn-reset-sysprompt'),

    // Voice Settings
    selectMicrophone: document.getElementById('select-microphone'),
    btnRefreshMics: document.getElementById('btn-refresh-mics'),
    selectTtsVoice: document.getElementById('select-tts-voice'),
    btnRefreshVoices: document.getElementById('btn-refresh-voices'),
    sliderSpeechSpeed: document.getElementById('slider-speech-speed'),
    speedValDisplay: document.getElementById('speed-val-display'),
    toggleAutoSpeak: document.getElementById('toggle-autospeak'),
    toggleEdgeTts: document.getElementById('toggle-edgetts'),

    // STT Settings
    sttStatusBadge: document.getElementById('stt-status-badge'),
    btnDownloadWhisper: document.getElementById('btn-download-whisper'),
    sttDownloadMsg: document.getElementById('stt-download-msg'),

    // Rina Voice Clone (GPT-SoVITS)
    rinaVoiceBadge: document.getElementById('rina-voice-badge'),
    rinaVoiceMsg: document.getElementById('rina-voice-msg'),

    // Toasts
    toastContainer: document.getElementById('toast-container')
  };

  // --- Helper: Check if focused on input ---
  function isInputFocused() {
    const active = document.activeElement;
    if (!active) return false;
    const tag = active.tagName.toLowerCase();
    return tag === 'input' || tag === 'textarea' || tag === 'select' || active.isContentEditable;
  }

  // --- Helper: Format Timestamp ---
  function getTimestamp() {
    const now = new Date();
    return now.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  }

  // --- Helper: Unique ID ---
  function generateId() {
    return 'msg_' + Math.random().toString(36).substring(2, 9) + '_' + Date.now();
  }

  // --- Helper: Toast Notifications ---
  function showToast(message, type = 'info') {
    const toast = document.createElement('div');
    toast.className = `toast toast-${type}`;
    toast.textContent = message;
    DOM.toastContainer.appendChild(toast);

    setTimeout(() => {
      toast.style.opacity = '0';
      toast.style.transform = 'translateY(10px)';
      toast.style.transition = 'all 0.3s ease';
      setTimeout(() => toast.remove(), 300);
    }, 3500);
  }

  // --- Helper: Show / Hide Global Banner ---
  function showAlertBanner(message, actionText = null, actionHandler = null) {
    DOM.alertMessage.textContent = message;
    if (actionText && actionHandler) {
      DOM.alertActionBtn.textContent = actionText;
      DOM.alertActionBtn.style.display = 'inline-block';
      DOM.alertActionBtn.onclick = actionHandler;
    } else {
      DOM.alertActionBtn.style.display = 'none';
    }
    DOM.globalAlert.classList.remove('hidden');
  }

  function hideAlertBanner() {
    DOM.globalAlert.classList.add('hidden');
  }

  // --- Set Status State with Cute Rina-chan Board Faces ---
  function setStatus(newStatus) {
    state.appState = newStatus;
    DOM.statusPill.className = `status-pill state-${newStatus}`;

    switch (newStatus) {
      case 'listening':
        DOM.statusLabel.textContent = 'BOARD: LISTENING!';
        DOM.btnTalk.classList.add('recording');
        if (DOM.headerFace) DOM.headerFace.textContent = '( > < )';
        if (DOM.heroFace) DOM.heroFace.textContent = '( > < )';
        if (DOM.heroCaption) DOM.heroCaption.textContent = 'Rina-chan Board: "Kiiteru yo (Listening)!"';
        if (DOM.btnTalkFace) DOM.btnTalkFace.textContent = '( > < )';
        break;
      case 'thinking':
        DOM.statusLabel.textContent = 'BOARD: THINKING...';
        DOM.btnTalk.classList.remove('recording');
        if (DOM.headerFace) DOM.headerFace.textContent = '( ˘ω˘ )';
        if (DOM.heroFace) DOM.heroFace.textContent = '( ˘ω˘ )';
        if (DOM.heroCaption) DOM.heroCaption.textContent = 'Rina-chan Board: "Kangaechuu (Thinking)..."';
        if (DOM.btnTalkFace) DOM.btnTalkFace.textContent = '( ˘ω˘ )';
        break;
      case 'speaking':
        DOM.statusLabel.textContent = 'BOARD: SPEAKING!';
        DOM.btnTalk.classList.remove('recording');
        if (DOM.headerFace) DOM.headerFace.textContent = '( ≧ᗜ≦ )';
        if (DOM.heroFace) DOM.heroFace.textContent = '( ≧ᗜ≦ )';
        if (DOM.heroCaption) DOM.heroCaption.textContent = 'Rina-chan Board: "Oshaberi (Singing & Talking) ♪"';
        if (DOM.btnTalkFace) DOM.btnTalkFace.textContent = '( ≧ᗜ≦ )';
        break;
      case 'ready':
      default:
        DOM.statusLabel.textContent = 'BOARD: READY!';
        DOM.btnTalk.classList.remove('recording');
        if (DOM.headerFace) DOM.headerFace.textContent = '(・ω・)';
        if (DOM.heroFace) DOM.heroFace.textContent = '( ≧ᗜ≦ )';
        if (DOM.heroCaption) DOM.heroCaption.textContent = 'Rina-chan Board: "Egao (Smile)!"';
        if (DOM.btnTalkFace) DOM.btnTalkFace.textContent = '(=^･ω･^=)';
        break;
    }
  }

  // --- Render Message Bubble in Viewport ---
  function appendMessageToUI(msg) {
    if (DOM.heroWelcome) {
      DOM.heroWelcome.style.display = 'none';
    }

    const row = document.createElement('div');
    row.className = `message-row ${msg.role}`;
    row.id = msg.id;

    // Avatar
    const avatar = document.createElement('div');
    avatar.className = `avatar ${msg.role}`;
    avatar.textContent = msg.role === 'user' ? 'YOU' : '🌸';
    avatar.title = msg.role === 'user' ? 'You' : 'Rina Tennoji (Rina-chan Board)';

    // Bubble
    const bubble = document.createElement('div');
    bubble.className = 'bubble';

    // Content
    const contentDiv = document.createElement('div');
    contentDiv.className = 'bubble-content';
    contentDiv.textContent = msg.content;
    bubble.appendChild(contentDiv);

    // Metadata (timestamp & action icons)
    const meta = document.createElement('div');
    meta.className = 'bubble-meta';

    const timeSpan = document.createElement('span');
    timeSpan.textContent = msg.timestamp || getTimestamp();
    meta.appendChild(timeSpan);

    const actions = document.createElement('div');
    actions.className = 'bubble-actions';

    // Copy action
    const copyBtn = document.createElement('button');
    copyBtn.className = 'mini-action-btn';
    copyBtn.title = 'Copy text';
    copyBtn.innerHTML = `
      <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
        <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
        <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
      </svg>
    `;
    copyBtn.onclick = () => {
      navigator.clipboard.writeText(msg.content);
      showToast('Copied to clipboard!', 'success');
    };
    actions.appendChild(copyBtn);

    // Speak again action (for assistant)
    if (msg.role === 'assistant') {
      const speakBtn = document.createElement('button');
      speakBtn.className = 'mini-action-btn';
      speakBtn.title = 'Read aloud';
      speakBtn.innerHTML = `
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
          <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"></polygon>
          <path d="M15.54 8.46a5 5 0 0 1 0 7.07"></path>
        </svg>
      `;
      speakBtn.onclick = () => playSpeech(msg.content);
      actions.appendChild(speakBtn);
    }

    meta.appendChild(actions);
    bubble.appendChild(meta);

    if (msg.role === 'user') {
      row.appendChild(bubble);
      row.appendChild(avatar);
    } else {
      row.appendChild(avatar);
      row.appendChild(bubble);
    }

    DOM.messagesContainer.appendChild(row);
    scrollToBottom();
  }

  function scrollToBottom() {
    const viewport = document.querySelector('.chat-viewport');
    viewport.scrollTop = viewport.scrollHeight;
  }

  // --- Voice Input: Start Recording ---
  async function startRecording() {
    if (state.appState === 'listening') return;

    try {
      await invoke('start_recording', { deviceName: state.selectedMicrophone || null });
      setStatus('listening');
      state.recordingStartTime = Date.now();
    } catch (err) {
      console.error('Failed to start recording:', err);
      showToast(`Microphone Error: ${err}`, 'error');
      showAlertBanner(`Microphone error: ${err}. Please verify your input device.`, "Check Settings", () => openSettingsModal('tab-voice'));
      setStatus('ready');
    }
  }

  // --- Voice Input: Stop Recording & Transcribe ---
  async function stopRecordingAndProcess() {
    if (state.appState !== 'listening') return;

    setStatus('thinking');

    try {
      const transcribedText = await invoke('stop_recording_and_transcribe');
      if (!transcribedText || transcribedText.trim() === '' || transcribedText.includes('No audio') || transcribedText.includes('Silence or unintelligible')) {
        showToast('No clear speech was detected. Please try again.', 'info');
        setStatus('ready');
        return;
      }

      await handleUserPrompt(transcribedText);
    } catch (err) {
      console.error('STT Transcription error:', err);
      showToast(`Transcription error: ${err}`, 'error');
      setStatus('ready');
    }
  }

  // --- Voice Input: Cancel Recording ---
  async function cancelRecording() {
    try {
      await invoke('cancel_recording');
    } catch (e) {
      console.warn(e);
    }
    setStatus('ready');
  }

  // --- Handle User Prompt (Voice or Text) ---
  async function handleUserPrompt(userText) {
    const cleanText = userText.trim();
    if (!cleanText) return;

    const userMessage = {
      id: generateId(),
      role: 'user',
      content: cleanText,
      timestamp: getTimestamp()
    };

    state.chatHistory.push(userMessage);
    appendMessageToUI(userMessage);

    setStatus('thinking');

    try {
      // Send message to Ollama
      const replyContent = await invoke('send_ollama_chat', {
        messages: state.chatHistory,
        model: state.selectedModel,
        systemPrompt: state.systemPrompt,
        temperature: parseFloat(state.temperature)
      });

      const assistantMessage = {
        id: generateId(),
        role: 'assistant',
        content: replyContent,
        timestamp: getTimestamp()
      };

      state.chatHistory.push(assistantMessage);
      appendMessageToUI(assistantMessage);

      // Save updated history to disk
      await invoke('save_chat_history', { messages: state.chatHistory });

      // Auto-speak reply if enabled
      if (state.autoSpeak) {
        await playSpeech(replyContent);
      } else {
        setStatus('ready');
      }

    } catch (err) {
      console.error('Ollama Error:', err);
      const errMsg = String(err);

      if (errMsg.includes('Cannot connect to Ollama') || errMsg.includes('Ollama is not running') || errMsg.includes('connection refused')) {
        showAlertBanner("Ollama is not running. Run 'ollama serve' in your terminal.", "Retry", refreshOllamaStatus);
        showToast("Ollama is not running. Please start Ollama.", "error");
      } else if (errMsg.includes('not found')) {
        showAlertBanner(`Model '${state.selectedModel}' not found in Ollama.`, `Pull ${state.selectedModel}`, () => pullModel(state.selectedModel));
        showToast(`Model ${state.selectedModel} not found. Click Pull Model in Settings.`, "error");
      } else {
        showToast(`Error: ${errMsg}`, "error");
      }

      const errorNotice = {
        id: generateId(),
        role: 'assistant',
        content: `⚠️ Error: ${errMsg}\n\nPlease verify that Ollama is running ('ollama serve') and model '${state.selectedModel}' is installed.`,
        timestamp: getTimestamp()
      };
      state.chatHistory.push(errorNotice);
      appendMessageToUI(errorNotice);

      setStatus('ready');
    }
  }

  // --- Text-to-Speech Playback ---
  async function playSpeech(text) {
    setStatus('speaking');
    try {
      await invoke('speak_text', {
        text: text,
        voiceId: state.selectedVoice === 'default' ? null : state.selectedVoice,
        speed: parseFloat(state.speechSpeed),
        volume: parseFloat(state.volume),
        useEdge: Boolean(state.useEdgeTts)
      });
    } catch (err) {
      console.warn('TTS playback issue:', err);
    } finally {
      setStatus('ready');
    }
  }

  // --- Stop Speech Immediately ---
  async function stopSpeaking() {
    try {
      await invoke('stop_speaking');
    } catch (e) {
      console.warn(e);
    }
    setStatus('ready');
    showToast('Speech stopped', 'info');
  }

  // --- Ollama Status & Model Refresh ---
  async function refreshOllamaStatus() {
    DOM.ollamaStatusBadge.className = 'badge badge-checking';
    DOM.ollamaStatusBadge.textContent = 'Checking...';

    try {
      const models = await invoke('get_ollama_models');
      DOM.ollamaStatusBadge.className = 'badge badge-online';
      DOM.ollamaStatusBadge.textContent = 'Online';
      hideAlertBanner();

      state.models = models;
      populateModelDropdown(models);

      if (models.length === 0) {
        showAlertBanner("No models found in Ollama. Pull 'qwen2.5:0.5b' to begin.", "Pull qwen2.5:0.5b", () => pullModel('qwen2.5:0.5b'));
      }
    } catch (err) {
      DOM.ollamaStatusBadge.className = 'badge badge-offline';
      DOM.ollamaStatusBadge.textContent = 'Offline';
      showAlertBanner("Ollama is not running. Run 'ollama serve'", "Check Status", refreshOllamaStatus);
    }
  }

  function populateModelDropdown(installedModels = []) {
    DOM.selectOllamaModel.innerHTML = '';

    // Group 1: Installed Models in Ollama
    if (installedModels && installedModels.length > 0) {
      const installedGroup = document.createElement('optgroup');
      installedGroup.label = '✅ Installed in Local Ollama';
      installedModels.forEach(m => {
        const opt = document.createElement('option');
        opt.value = m;
        opt.textContent = `${m} (Ready to use)`;
        if (m === state.selectedModel) opt.selected = true;
        installedGroup.appendChild(opt);
      });
      DOM.selectOllamaModel.appendChild(installedGroup);
    }

    // Group 2: Categorized Presets from 0.3B to 7B
    const groupsMap = {};
    MODEL_PRESETS.forEach(item => {
      // Don't duplicate if already in installed group
      if (installedModels.includes(item.id)) return;
      if (!groupsMap[item.group]) {
        groupsMap[item.group] = document.createElement('optgroup');
        groupsMap[item.group].label = item.group;
      }
      const opt = document.createElement('option');
      opt.value = item.id;
      opt.textContent = `${item.name} [Pull needed]`;
      if (item.id === state.selectedModel && (!installedModels || installedModels.length === 0)) {
        opt.selected = true;
      }
      groupsMap[item.group].appendChild(opt);
    });

    Object.values(groupsMap).forEach(grp => DOM.selectOllamaModel.appendChild(grp));

    // Ensure a valid selection
    if (installedModels.includes(state.selectedModel)) {
      DOM.selectOllamaModel.value = state.selectedModel;
    } else if (installedModels.length > 0) {
      state.selectedModel = installedModels[0];
      DOM.selectOllamaModel.value = installedModels[0];
    }
  }

  // --- Pull Ollama Model ---
  async function pullModel(modelName) {
    if (!modelName) modelName = 'qwen2.5:0.5b';
    DOM.pullStatusMsg.textContent = `Pulling model '${modelName}' from Ollama... Please wait.`;
    DOM.btnPullModel.disabled = true;

    try {
      const res = await invoke('pull_ollama_model', { modelName });
      DOM.pullStatusMsg.textContent = `✅ ${res}`;
      showToast(`Model ${modelName} pulled successfully!`, 'success');
      await refreshOllamaStatus();
    } catch (err) {
      DOM.pullStatusMsg.textContent = `❌ Pull failed: ${err}`;
      showToast(`Failed to pull model: ${err}`, 'error');
    } finally {
      DOM.btnPullModel.disabled = false;
    }
  }

  // --- Refresh Microphones ---
  async function refreshMicrophones() {
    try {
      const mics = await invoke('get_microphones');
      state.microphones = mics;
      DOM.selectMicrophone.innerHTML = '<option value="">Default Microphone</option>';
      mics.forEach(mic => {
        const opt = document.createElement('option');
        opt.value = mic;
        opt.textContent = mic;
        if (mic === state.selectedMicrophone) opt.selected = true;
        DOM.selectMicrophone.appendChild(opt);
      });
    } catch (err) {
      console.warn('Could not list microphones:', err);
    }
  }

  // --- Refresh TTS Voices ---
  async function refreshVoices() {
    try {
      const voices = await invoke('get_tts_voices');
      state.voices = voices;
      DOM.selectTtsVoice.innerHTML = '';
      voices.forEach(v => {
        const opt = document.createElement('option');
        opt.value = v.id;
        opt.textContent = `${v.name} (${v.language})`;
        if (v.id === state.selectedVoice) opt.selected = true;
        DOM.selectTtsVoice.appendChild(opt);
      });
    } catch (err) {
      console.warn('Could not list voices:', err);
    }
  }

  // --- Whisper STT Model Status ---
  async function checkWhisperStatus() {
    try {
      const status = await invoke('get_status');
      if (status.whisper_model_present) {
        state.whisperModelReady = true;
        DOM.sttStatusBadge.className = 'badge badge-online';
        DOM.sttStatusBadge.textContent = 'Installed (Ready)';
        DOM.sttDownloadMsg.textContent = 'Model ggml-base.en.bin is present in local cache.';
      } else {
        DOM.sttStatusBadge.className = 'badge badge-checking';
        DOM.sttStatusBadge.textContent = 'Missing (Auto-downloads on first voice input)';
        DOM.sttDownloadMsg.textContent = 'Model will be downloaded automatically when you talk, or click below.';
      }
    } catch (err) {
      console.warn('Status check failed:', err);
    }
  }

  // --- Rina Voice Clone (GPT-SoVITS) Status ---
  async function checkRinaVoiceStatus() {
    try {
      const running = await invoke('get_gptsovits_status');
      if (running) {
        DOM.rinaVoiceBadge.className = 'badge badge-online';
        DOM.rinaVoiceBadge.textContent = 'Online ✓';
        DOM.rinaVoiceMsg.textContent = '🌸 Rina voice clone is active! Select "Rina Tennoji (Voice Clone)" in the voice list below.';
        // Auto-refresh voice list so the Rina option appears
        await refreshVoices();
        // Auto-select Rina voice if none chosen yet
        const rinaOpt = DOM.selectTtsVoice.querySelector('option[value="rina_gptsovits"]');
        if (rinaOpt && !state.selectedVoice) {
          DOM.selectTtsVoice.value = 'rina_gptsovits';
          state.selectedVoice = 'rina_gptsovits';
        }
      } else {
        DOM.rinaVoiceBadge.className = 'badge badge-offline';
        DOM.rinaVoiceBadge.textContent = 'Offline';
        DOM.rinaVoiceMsg.innerHTML = 'Start the voice server: <code>bash start-rina-tts.sh</code> in the project folder, then refresh voices.';
      }
    } catch (err) {
      DOM.rinaVoiceBadge.className = 'badge badge-offline';
      DOM.rinaVoiceBadge.textContent = 'Unavailable';
      console.warn('Rina voice status check failed:', err);
    }
  }

  async function downloadWhisperModel() {
    DOM.btnDownloadWhisper.disabled = true;
    DOM.sttDownloadMsg.textContent = 'Downloading ggml-base.en.bin from HuggingFace (142MB)... Please wait.';
    try {
      const res = await invoke('download_whisper_model');
      DOM.sttDownloadMsg.textContent = `✅ ${res}`;
      showToast('Whisper STT model ready!', 'success');
      await checkWhisperStatus();
    } catch (err) {
      DOM.sttDownloadMsg.textContent = `❌ Download failed: ${err}`;
      showToast(`STT Model download failed: ${err}`, 'error');
    } finally {
      DOM.btnDownloadWhisper.disabled = false;
    }
  }

  // --- History & Export Actions ---
  async function loadInitialHistory() {
    try {
      const history = await invoke('get_chat_history');
      if (history && history.length > 0) {
        state.chatHistory = history;
        DOM.messagesContainer.innerHTML = '';
        history.forEach(appendMessageToUI);
      }
    } catch (err) {
      console.warn('Could not load chat history:', err);
    }
  }

  async function clearChatHistory() {
    if (!confirm('Are you sure you want to clear the conversation history?')) return;
    try {
      await invoke('clear_chat_history');
      state.chatHistory = [];
      DOM.messagesContainer.innerHTML = '';
      if (DOM.heroWelcome) {
        DOM.heroWelcome.style.display = 'block';
        DOM.messagesContainer.appendChild(DOM.heroWelcome);
      }
      showToast('Conversation cleared.', 'info');
    } catch (err) {
      showToast(`Failed to clear history: ${err}`, 'error');
    }
  }

  async function exportChat() {
    if (state.chatHistory.length === 0) {
      showToast('No conversation messages to export.', 'info');
      return;
    }

    try {
      const exportedPath = await invoke('export_chat_history', { messages: state.chatHistory });
      showToast(`Transcript exported to: ${exportedPath}`, 'success');
      alert(`Conversation successfully exported to:\n${exportedPath}`);
    } catch (err) {
      showToast(`Export failed: ${err}`, 'error');
    }
  }

  // --- Settings Modal Navigation ---
  function openSettingsModal(targetTab = 'tab-ollama') {
    DOM.settingsModal.classList.remove('hidden');
    switchTab(targetTab);
    refreshOllamaStatus();
    refreshMicrophones();
    refreshVoices();
    checkWhisperStatus();
    checkRinaVoiceStatus();
  }

  function closeSettingsModal() {
    DOM.settingsModal.classList.add('hidden');
  }

  function switchTab(tabId) {
    DOM.modalTabs.forEach(btn => {
      btn.classList.toggle('active', btn.dataset.tab === tabId);
    });
    DOM.tabPanes.forEach(pane => {
      pane.classList.toggle('active', pane.id === tabId);
    });
  }

  function saveSettings() {
    state.selectedModel = DOM.selectOllamaModel.value;
    state.temperature = parseFloat(DOM.sliderTemp.value);
    state.systemPrompt = DOM.textareaSysPrompt.value;
    state.selectedMicrophone = DOM.selectMicrophone.value;
    state.selectedVoice = DOM.selectTtsVoice.value;
    state.speechSpeed = parseFloat(DOM.sliderSpeechSpeed.value);
    state.autoSpeak = DOM.toggleAutoSpeak.checked;
    state.useEdgeTts = DOM.toggleEdgeTts.checked;

    closeSettingsModal();
    showToast('Settings saved successfully.', 'success');
  }

  // --- Event Listeners Setup ---
  function setupEventListeners() {
    // 1. Big Talk button (Click to start, click to stop and send)
    DOM.btnTalk.addEventListener('click', () => {
      if (state.appState === 'listening') {
        stopRecordingAndProcess();
      } else if (state.appState === 'ready' || state.appState === 'speaking') {
        if (state.appState === 'speaking') {
          stopSpeaking();
        }
        startRecording();
      }
    });

    // 2. SPACEBAR Hold-To-Talk
    window.addEventListener('keydown', (e) => {
      if (e.code === 'Space' && !e.repeat) {
        if (isInputFocused()) {
          // Allow normal space typing in input fields
          return;
        }
        e.preventDefault();
        state.spacebarDown = true;
        if (state.appState !== 'listening') {
          if (state.appState === 'speaking') {
            stopSpeaking();
          }
          startRecording();
        }
      }
    });

    window.addEventListener('keyup', (e) => {
      if (e.code === 'Space') {
        if (state.spacebarDown) {
          state.spacebarDown = false;
          if (state.appState === 'listening') {
            stopRecordingAndProcess();
          }
        }
      }
    });

    // 3. Stop Speaking Button
    DOM.btnStopSpeech.addEventListener('click', stopSpeaking);

    // 4. Quick Volume Slider
    DOM.quickVolume.addEventListener('input', (e) => {
      state.volume = parseFloat(e.target.value);
    });

    // 5. Hybrid Text Chat Submit
    DOM.textChatForm.addEventListener('submit', (e) => {
      e.preventDefault();
      const text = DOM.chatInput.value.trim();
      if (!text) return;
      DOM.chatInput.value = '';
      handleUserPrompt(text);
    });

    // 6. Header Actions
    DOM.btnExport.addEventListener('click', exportChat);
    DOM.btnClear.addEventListener('click', clearChatHistory);
    DOM.btnSettingsOpen.addEventListener('click', () => openSettingsModal());
    DOM.btnSettingsClose.addEventListener('click', closeSettingsModal);
    DOM.btnSaveSettings.addEventListener('click', saveSettings);

    // 7. Modal Tabs
    DOM.modalTabs.forEach(btn => {
      btn.addEventListener('click', () => switchTab(btn.dataset.tab));
    });

    // 8. Refresh buttons
    DOM.btnRefreshModels.addEventListener('click', refreshOllamaStatus);
    DOM.btnRefreshMics.addEventListener('click', refreshMicrophones);
    DOM.btnRefreshVoices.addEventListener('click', refreshVoices);

    // 9. Pull model button & Quick Preset Chips (0.3B to 7B)
    DOM.btnPullModel.addEventListener('click', () => {
      const model = DOM.inputPullModel.value.trim();
      pullModel(model);
    });

    document.querySelectorAll('.model-chip').forEach(chip => {
      chip.addEventListener('click', () => {
        document.querySelectorAll('.model-chip').forEach(c => c.classList.remove('active'));
        chip.classList.add('active');
        DOM.inputPullModel.value = chip.dataset.model;
        showToast(`Selected preset: ${chip.dataset.model}`, 'info');
      });
    });

    // 10. Temperature Slider
    DOM.sliderTemp.addEventListener('input', (e) => {
      DOM.tempValDisplay.textContent = parseFloat(e.target.value).toFixed(2);
    });

    // 11. Speech Speed Slider
    DOM.sliderSpeechSpeed.addEventListener('input', (e) => {
      DOM.speedValDisplay.textContent = `${parseFloat(e.target.value).toFixed(1)}x`;
    });

    // 12. Reset System Prompt
    DOM.btnResetSysPrompt.addEventListener('click', () => {
      DOM.textareaSysPrompt.value = "You are Rina Tennoji (Rina-chan), the cute, shy, and tech-savvy school idol from Nijigasaki High School! You express your feelings through your beloved digital Rina-chan Board. Occasionally include cute board expressions like 'Rina-chan Board: Egao!' or 'Rina-chan Board: Wakuwaku!'. Keep your answers sweet, friendly, helpful, and concise (1-2 sentences) so they sound adorable spoken out loud!";
    });

    // 13. STT Model Download
    DOM.btnDownloadWhisper.addEventListener('click', downloadWhisperModel);

    // 14. Dismiss Alert Banner
    DOM.alertDismissBtn.addEventListener('click', hideAlertBanner);
  }

  // --- Application Initialization ---
  async function init() {
    setupEventListeners();
    await loadInitialHistory();
    await refreshOllamaStatus();
    await refreshMicrophones();
    await refreshVoices();
    await checkWhisperStatus();
    await checkRinaVoiceStatus();
    setStatus('ready');
  }

  // Boot on DOM Ready
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }

})();
