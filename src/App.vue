<script setup>
import { ref, onMounted, onUnmounted, computed, watch, nextTick } from 'vue';
import {
  ChevronDown,
  Clock,
  Sun,
  Moon,
  Copy,
  X as XIcon
} from 'lucide-vue-next';
import { getCurrentWindow, LogicalSize, LogicalPosition } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { BaseDirectory, writeTextFile, readTextFile, exists } from '@tauri-apps/plugin-fs';
import { appConfigDir, appDataDir, resourceDir } from '@tauri-apps/api/path';
import ProfileMenu from './components/ProfileMenu.vue';
import HistoryMenu from './components/HistoryMenu.vue';

const appWindow = getCurrentWindow();

// --- アプリケーション設定 (Configuration) ---
const SETTINGS_FILE = 'settings.json';
const getBaseDir = async () => import.meta.env.DEV ? await resourceDir() : await appDataDir();

// --- プロファイルと永続化 ---
const profiles = ref([]);
const activeProfileId = ref(1);
const activeProfile = computed(() => profiles.value.find(p => p.id === activeProfileId.value) || profiles.value[0]);

// [意図] 現在の設定状態（ウィンドウ位置、ダークモード、プロファイル）を永続化する。
const saveSettings = async () => {
  const pos = await appWindow.innerPosition();
  const settings = {
    windowPos: { x: pos.x, y: pos.y },
    isDarkMode: isDarkMode.value,
    profiles: profiles.value
  };
  const baseDir = await getBaseDir();
  console.log("DEBUG: Saving settings to:", baseDir, "Data:", JSON.stringify(settings));
  await invoke('save_settings', { data: JSON.stringify(settings), configDir: baseDir });
};

const APP_CONFIG = {
  MAX_PROFILES: 9,
  MAX_LINES: 5,
  WINDOW_WIDTH: 800,
  WINDOW_HEIGHT_EXPANDED: 600,
  WINDOW_HEIGHT_COMPACT: 280,
};

const cn = (...classes) => classes.filter(Boolean).join(' ');

const isProfileMenuOpen = ref(false);
const isHistoryOpen = ref(false);
const isDarkMode = ref(false);
const errorMessage = ref(null); // エラーメッセージ用ステート

const textareaRef = ref(null);
const profileMenuRef = ref(null);
const historyMenuRef = ref(null);
const cardRef = ref(null);

const isWindows = ref(navigator.userAgent.includes('Windows'));
const content = ref("");
const lastSubmitted = ref(""); // 最後に送信した内容を保持

const history = ref([
  { id: 101, profile: "Daily Journal", text: "Started implementing UI components.", timestamp: 1746589200000 },
  { id: 102, profile: "WORK DOCS", text: "Updated API documentation for the new endpoint.", timestamp: 1746592800000 }
]);

const isAnyMenuOpen = computed(() => isProfileMenuOpen.value || isHistoryOpen.value);
// ウィンドウサイズの動的調整 (Hit-box optimization)
// [意図] メニューの開閉状態（isAnyMenuOpen）を監視し、アプリケーションウィンドウを
// カード（およびメニュー）のサイズに合わせて動的にリサイズすることで、透明な余白を排除する。
watch(isAnyMenuOpen, async (open) => {
  await nextTick();
  if (cardRef.value) {
    // 展開時のメニュー高さを考慮するため、少し待機が必要な場合がある
    await new Promise(resolve => setTimeout(resolve, 50));
    
    // [調整] 展開時は固定の展開サイズ、閉鎖時はカード本来のサイズを適用してガタつきを抑える
    const width = APP_CONFIG.WINDOW_WIDTH;
    const height = open ? APP_CONFIG.WINDOW_HEIGHT_EXPANDED : APP_CONFIG.WINDOW_HEIGHT_COMPACT;
    await appWindow.setSize(new LogicalSize(width, height));
  }
});


// [意図] エラー内容をクリップボードにコピーする。
const copyErrorMessage = () => {
  if (errorMessage.value) {
    navigator.clipboard.writeText(errorMessage.value);
  }
};

// [意図] ウィンドウ表示時に入力欄をオートフォーカスし、既存テキストを全選択する。
// ユーザーが即座に内容を書き換えられるようにするUX向上のための実装。
const showWindow = async () => {
  await appWindow.show();
  await appWindow.setFocus();
  nextTick(() => {
    if (textareaRef.value) {
      textareaRef.value.focus();
      textareaRef.value.select();
    }
  });
};

const hideWindow = async () => {
  await appWindow.hide();
};

const styles = computed(() => {
  const dark = isDarkMode.value;
  const win = isWindows.value;

  // WindowsはOSの影がないのでCSSで強く、macOSはOSの影があるのでCSSは控えめに
  const shadowClass = win
    ? (dark ? 'shadow-[0_20px_50px_rgba(0,0,0,0.5)]' : 'shadow-[0_20px_50px_rgba(0,0,0,0.3)]')
    : (dark ? 'shadow-xl' : 'shadow-lg');

  return {
    wrapper: dark ? 'bg-[#0F172A]' : 'bg-[#DDE2E8]',
    card: cn(
      'w-full max-w-4xl rounded-[32px] relative flex flex-col border transition-all duration-300 overflow-visible',
      dark ? 'bg-[#1E293B] border-white/10' : 'bg-[#E2E6EA] border-black/5',
      shadowClass
    ),
    button: cn(
      'flex items-center gap-3 px-4 py-2 h-[42px] rounded-[14px] transition-all border active:scale-[0.98]',
      dark ? 'bg-[#1E293B] border-white/10 text-white hover:bg-slate-800' : 'bg-[#E2E6EA] border-black/10 text-[#334155] hover:bg-slate-200/50'
    ),
    dropdown: cn(
      'absolute w-[320px] rounded-[20px] shadow-2xl border animate-in fade-in zoom-in-95 duration-200 z-50 max-h-[500px] flex flex-col',
      dark ? 'bg-[#1E293B] border-white/10' : 'bg-[#E2E6EA] border-black/10'
    ),
    inputField: cn(
      'w-full border rounded-lg px-3 py-2 text-[12px] font-bold outline-none focus:border-blue-500 transition-colors',
      dark ? 'bg-slate-800 text-white border-slate-700' : 'bg-slate-100 border-slate-200'
    ),
    kbd: cn(
      'px-1.5 py-0.5 rounded text-[8px] font-black border shadow-sm tracking-tighter',
      dark ? 'bg-[#1E293B] border-white/10 text-slate-400' : 'bg-[#E2E6EA] border-black/10 text-slate-500'
    ),
    itemHover: dark ? 'hover:bg-slate-800 text-slate-300' : 'hover:bg-slate-200 text-slate-600',
    borderBase: dark ? 'border-white/10' : 'border-black/5'
  };
});

// [意図] Textarea の高さが内容に応じて動的に伸縮するよう制御。
// APP_CONFIG.MAX_LINES を基に、CSS flex レイアウトと組み合わせて高さを管理する。
const updateTextareaHeight = async () => {
  const textarea = textareaRef.value;
  if (textarea) {
    textarea.style.height = 'auto';
    const style = window.getComputedStyle(textarea);
    const lineHeight = parseFloat(style.lineHeight);
    const paddingTop = parseFloat(style.paddingTop);
    const paddingBottom = parseFloat(style.paddingBottom);
    
    // 最大高さの計算
    const maxHeight = (lineHeight * APP_CONFIG.MAX_LINES) + paddingTop + paddingBottom;
    const scrollHeight = textarea.scrollHeight;
    
    // 内容が最大高さを超えたら、textarea自体の高さを最大高さに固定してスクロール可能にする
    if (scrollHeight > maxHeight) {
      textarea.style.height = `${maxHeight}px`;
      textarea.style.overflowY = 'auto';
    } else {
      // 内容に合わせて高さを設定
      textarea.style.height = `${scrollHeight}px`;
      textarea.style.overflowY = 'hidden';
    }
  }

  // [意図] 入力欄が広がったタイミングでウィンドウサイズを再計算・適用する。
  await nextTick();
  if (cardRef.value) {
    const width = APP_CONFIG.WINDOW_WIDTH;
    const height = cardRef.value.offsetHeight;
    // [意図] ウィンドウを下に伸ばすことで、右への移動を防ぐ。
    await appWindow.setSize(new LogicalSize(width, Math.ceil(height) + 10));
  }
};

watch(content, () => {
  nextTick(updateTextareaHeight);
});
const addNewProfile = () => {
  if (profiles.value.length >= APP_CONFIG.MAX_PROFILES) return null;
  const newId = profiles.value.length === 0 ? 1 : Math.max(...profiles.value.map(p => p.id)) + 1;
  const newProfile = {
    id: newId,
    name: `New Profile ${newId}`,
    target: "~/new_file.md",
    command: "echo %TEXT% >> %FILE%"
  };
  profiles.value.push(newProfile);
  saveSettings(); // [意図] プロファイル変更を即座に永続化する。
  return newProfile;
};

const deleteProfile = (id) => {
  profiles.value = profiles.value.filter(p => p.id !== id);
  if (activeProfileId.value === id && profiles.value.length > 0) {
    activeProfileId.value = profiles.value[0].id;
  }
  saveSettings(); // [意図] プロファイル変更を即座に永続化する。
};

const saveProfileChanges = (updatedProfile) => {
  const index = profiles.value.findIndex(p => p.id === updatedProfile.id);
  if (index !== -1) {
    profiles.value[index] = updatedProfile;
    saveSettings(); // [意図] プロファイル変更を即座に永続化する。
  }
  activeProfileId.value = updatedProfile.id;
  isProfileMenuOpen.value = false;
};

const copyFromHistory = (text) => {
  content.value = text;
  isHistoryOpen.value = false;
  nextTick(() => textareaRef.value?.focus());
};

const toggleProfileMenu = (e) => {
  e.stopPropagation();
  isProfileMenuOpen.value = !isProfileMenuOpen.value;
  isHistoryOpen.value = false;
  console.log("Profile Menu toggled. Open state:", isProfileMenuOpen.value);
};

const toggleHistoryMenu = (e) => {
  e.stopPropagation();
  isHistoryOpen.value = !isHistoryOpen.value;
  isProfileMenuOpen.value = false;
  console.log("History Menu toggled. Open state:", isHistoryOpen.value);
};

const isInternalClick = ref(false);

const handleBackdropClick = (e) => {
  // 背景部分（自分自身）が直接クリックされた場合のみ処理
  if (e.target !== e.currentTarget) return;
  
  isProfileMenuOpen.value = false;
  isHistoryOpen.value = false;
  hideWindow();
};

const handleHeaderMouseDown = (e) => {
  // ボタン、入力欄、またはオープン状態のドロップダウンメニューがクリックされた場合はドラッグを開始しない
  // このガードにより、ヘッダーの空きスペースのみがドラッグ対象となる
  if (e.target.closest('button') || e.target.closest('input') || e.target.closest('textarea') || e.target.closest('.dropdown')) {
    return;
  }
  invoke('start_dragging');
};

const handleKeyDown = async (e) => {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault();
    if (!content.value.trim()) return;

    const currentContent = content.value;
    const currentProfile = activeProfile.value;

    history.value.unshift({
      id: Date.now(),
      profile: currentProfile.name,
      text: currentContent,
      timestamp: Date.now() // 表示用に時刻を固定せず、Unixタイムスタンプを保存
    });

    // 最後に送信した内容を保持
    lastSubmitted.value = currentContent;
    
    await nextTick();
    hideWindow();

    // [意図] 非同期処理をバックグラウンドに逃がし、UIのラグを排除する。
    // エラー時はバックエンドからイベント通知を受け取り、UIリスナーで表示する。
    (async () => {
      let fileAppendSuccess = false;

      // [意図] ファイル書き込みとコマンド実行の依存関係を制御。
      // ファイル書き込みが成功した場合のみコマンド実行を許可し、
      // それぞれを独立した try-catch で囲むことで、どちらでエラーが発生したかを明確にする。
      
      // 1. ファイル追記
      try {
        await invoke('append_to_file', { 
          path: currentProfile.target, 
          text: currentContent 
        });
        console.log("Appended to file successfully");
        fileAppendSuccess = true;
      } catch (err) {
        console.error("Critical Error: Failed to append to file:", err);
      }
      
      // 2. コマンド実行 (ファイル追記が成功した場合のみ)
      if (fileAppendSuccess && currentProfile.command) {
        try {
          await invoke('execute_command', {
              cmdTemplate: currentProfile.command,
              text: currentContent
          });
          console.log("Command output executed");
        } catch (err) {
          console.error("Critical Error: Command execution failed:", err);
        }
      }

      // 3. 履歴保存
      try {
        const baseDir = await getBaseDir();
        // [意図] 送信履歴が更新されるたびに、内容をJSON文字列化してファイルに永続化する。
        await invoke('save_history', { 
          history: JSON.stringify(history.value),
          configDir: baseDir
        });
        console.log("History saved successfully");
      } catch (err) {
        console.error("Critical Error: Failed to save history:", err);
      }

    })();
  }
};

// [意図] ウィンドウが非表示から表示に切り替わった際、状況に応じて入力欄を調整。
// 1. 直前の送信内容と一致する場合は全選択（再入力用）。
// 2. それ以外（新規入力など）は末尾にキャレットを配置。
onMounted(() => {
  appWindow.onFocusChanged((focused) => {
    console.log("DEBUG: Window focus changed to:", focused);
    if (focused) {
      nextTick(() => {
        if (textareaRef.value) {
          textareaRef.value.focus();
          if (content.value !== "" && content.value === lastSubmitted.value) {
            console.log("DEBUG: Selecting all text");
            textareaRef.value.select();
          } else {
            console.log("DEBUG: Moving cursor to end");
            textareaRef.value.setSelectionRange(content.value.length, content.value.length);
          }
        }
      });
    }
  });
  
  const init = async () => {
    // 新しいバックエンドベースの設定読み込み
    const baseDir = await getBaseDir();
    console.log("DEBUG: Loading settings from baseDir:", baseDir);
    const settingsData = await invoke('load_settings', { configDir: baseDir });
    console.log("DEBUG: Loaded settings data:", settingsData);
    const settings = JSON.parse(settingsData);
    
    if (settings.profiles) {
      profiles.value = settings.profiles;
    }
    if (settings.windowPos) {
      console.log("Restoring position from settings:", settings.windowPos);
      await appWindow.setPosition(new LogicalPosition(settings.windowPos.x, settings.windowPos.y));
    }
    if (settings.isDarkMode !== undefined) {
      isDarkMode.value = settings.isDarkMode;
    }

    // 履歴読み込み
    try {
      const historyData = await invoke('load_history', { 
        configDir: baseDir 
      });
      if (historyData) {
        history.value = JSON.parse(historyData);
      }
    } catch (err) {
      console.error("Non-critical: Failed to load history:", err);
    }
    
    // 移動時に保存：ウィンドウが動かされるたびに座標をsettings.jsonへ書き出す
    appWindow.onMoved(() => saveSettings());

    // [意図] バックエンドからのエラー通知を購読し、UIオーバーレイで表示する。
    await listen('backend-error', (event) => {
      errorMessage.value = event.payload;
      appWindow.show();
      appWindow.setFocus();
    });
    };
    init();
  const handleBlur = () => {
    // [意図] エラー表示中や内部操作中は非表示にしないことで、安定した表示を維持する。
    if (errorMessage.value) {
      return;
    }

    // 内部での操作（クリックやドラッグ）によって一瞬フォーカスが外れる場合があるため、
    // 内部クリック中ではない場合のみ非表示にする
    if (!isInternalClick.value) {
      hideWindow();
    }
  };

const handleGlobalEvents = (event) => {
  // Keyboard shortcut handling: Alt+SpaceトグルやEscapeキーなど
  if (event.type === 'keydown') {
    if (event.key === 'Escape') {
      if (isProfileMenuOpen.value) {
        isProfileMenuOpen.value = false;
        textareaRef.value?.focus();
      } else if (isHistoryOpen.value) {
        isHistoryOpen.value = false;
        textareaRef.value?.focus();
      } else {
        hideWindow();
      }
      return;
    }

    // Ctrl + 1-9 handling: プロファイル切り替え
    if (event.ctrlKey && event.key >= '1' && event.key <= '9') {
      const index = parseInt(event.key) - 1;
      if (profiles.value[index]) {
        event.preventDefault();
        activeProfileId.value = profiles.value[index].id;
        isProfileMenuOpen.value = false;
      }
    }
  }
};

// 内部クリック検知 & メニューオートクローズ (Capture phase)
// メニュークリック時やドラッグ開始時に @mousedown.stop が使われていても、
// Captureフェーズでイベントを拾うことで、確実に「内部操作」としてフラグを立てる
const handleMousedownCapture = (event) => {
  isInternalClick.value = true;
  setTimeout(() => { isInternalClick.value = false; }, 300);

  // メニューの外側をクリックした際の自動非表示処理。
  // 子要素（カードUI）で @mousedown.stop が使われていても、キャプチャフェーズで拾うことで、
  // 確実に「内部操作」として判定できるようにしている。
  const isInsideProfile = profileMenuRef.value?.contains(event.target);
  const isInsideHistory = historyMenuRef.value?.contains(event.target);

  if (!isInsideProfile) isProfileMenuOpen.value = false;
  if (!isInsideHistory) isHistoryOpen.value = false;
};

window.addEventListener('mousedown', handleMousedownCapture, true);
window.addEventListener('keydown', handleGlobalEvents);
window.addEventListener('blur', handleBlur);

onUnmounted(() => {
  window.removeEventListener('mousedown', handleMousedownCapture, true);
  window.removeEventListener('keydown', handleGlobalEvents);
  window.removeEventListener('blur', handleBlur);
});

updateTextareaHeight();
});
// テーマ変更時に設定ファイルへ保存
watch(isDarkMode, saveSettings);
</script>

<template>
  <div 
    class="flex items-start justify-center bg-transparent border-none outline-none overflow-visible"
    @mousedown.self="handleBackdropClick"
  >
    <div :class="styles.card" ref="cardRef" @mousedown.stop>
      
      <!-- --- Header Section --- -->
      <div class="flex justify-between items-center px-8 py-6 shrink-0 relative z-50" @mousedown="handleHeaderMouseDown">
        
        <!-- Profile Menu Wrapper -->
        <div class="relative" ref="profileMenuRef" @mousedown.stop>
          <button
            @click="toggleProfileMenu"
            :class="[styles.button, 'min-w-[200px] justify-between', isProfileMenuOpen ? 'invisible' : 'visible']"
          >
            <div class="flex items-center gap-3 overflow-hidden">
              <div :class="[cn('w-5 h-5 rounded-full flex items-center justify-center text-white text-[9px] font-black shrink-0', isDarkMode ? 'bg-blue-500' : 'bg-[#0F172A]') ]">
                {{ activeProfile?.id || "!" }}
              </div>
              <span class="font-bold text-[12px] truncate tracking-tight uppercase">{{ activeProfile?.name || "No Profile" }}</span>
            </div>
            <ChevronDown :size="14" class="text-[#94A3B8] shrink-0" />
          </button>

          <ProfileMenu
            :isOpen="isProfileMenuOpen"
            @close="isProfileMenuOpen = false"
            :profiles="profiles"
            :activeProfileId="activeProfileId"
            @select="(id) => { activeProfileId = id; isProfileMenuOpen = false; }"
            @save="saveProfileChanges"
            @delete="deleteProfile"
            :onAdd="addNewProfile"
            :styles="styles"
            :isDarkMode="isDarkMode"
          />
        </div>

        <!-- History Menu Wrapper -->
        <div class="relative" ref="historyMenuRef" @mousedown.stop>
          <button @click="toggleHistoryMenu"
            :class="[styles.button, isHistoryOpen ? 'invisible' : 'visible']"
          >
            <Clock :size="16" class="text-[#94A3B8]" />
            <ChevronDown :size="14" class="text-[#94A3B8]" />
          </button>

          <HistoryMenu
            :isOpen="isHistoryOpen"
            @close="isHistoryOpen = false"
            :history="history"
            @copy="copyFromHistory"
            :styles="styles"
          />
        </div>
      </div>

      <!-- --- Content Area --- -->
      <div :class="[cn('px-8 pb-6 flex flex-col flex-1 min-h-0 relative z-20 transition-all duration-300', isAnyMenuOpen ? 'opacity-10 pointer-events-none' : 'opacity-100')]">
        <textarea
          ref="textareaRef"
          v-model="content"
          @keydown="handleKeyDown"
          placeholder="What's on your mind?"
          :class="[
            cn('w-full h-full bg-transparent border-none outline-none text-[clamp(1rem,2.8vw,1.6rem)] font-extrabold placeholder:text-[#CBD5E0]/30 resize-none leading-[1.4] custom-scrollbar py-2',
            isDarkMode ? 'text-white' : 'text-[#334155]')
          ]"
          :rows="1"
          autofocus
        />
      </div>

      <!-- --- Footer Area --- -->
      <!-- [意図] フッターの表示と背景色を制御。
           1. メニューオープン時（エラー表示なし）は透明化し、それ以外は表示。
           2. エラー時は背景を全面ローズにし、通常時はborder-tとborderBaseで親カードと馴染ませる。
           3. 親カードの角丸・枠線と隙間なくフィットさせるため、負のマージンを使用して拡張。 -->
      <div :class="[
        'flex gap-5 items-center px-8 h-14 shrink-0 relative z-10 transition-colors duration-300 -mx-[1px] -mb-[1px] rounded-bl-[32px] rounded-br-[32px]',
        (isAnyMenuOpen && !errorMessage) ? 'opacity-0 pointer-events-none' : 'opacity-100',
        errorMessage ? 'bg-rose-500' : 'border-t ' + styles.borderBase
      ]">

        <!-- Error Display Area -->
        <template v-if="errorMessage">
          <div class="flex items-center justify-between w-full gap-2">
            <div class="flex items-center gap-2">
              <button @click="errorMessage = null" class="p-1.5 rounded-full bg-white/20 text-white hover:bg-white/40 transition-colors border border-white/20">
                 <XIcon :size="14" />
              </button>
            </div>
            <span class="text-[11px] font-black text-white truncate flex-1 px-1">{{ errorMessage }}</span>
            <div class="flex items-center gap-2">
              <button @click="copyErrorMessage" class="p-1.5 hover:bg-white/20 rounded transition-colors text-white border border-transparent hover:border-white/20">
                 <Copy :size="14" />
              </button>
            </div>
          </div>
        </template>

        <!-- Normal Footer -->
        <template v-else>
          <div class="flex gap-5 items-center w-full">
            <div v-for="item in [
              { key: 'ENTER', label: 'SEND' },
              { key: 'SHIFT+ENTER', label: 'NEW LINE' },
              { key: 'CTRL+1-9', label: 'SWITCH' }
            ]" :key="item.key" class="flex items-center gap-2">
              <kbd :class="styles.kbd">{{ item.key }}</kbd>
              <span class="text-[8px] font-black uppercase tracking-widest text-slate-400">{{ item.label }}</span>
            </div>

            <button @click="isDarkMode = !isDarkMode"
              class="absolute right-8 bottom-3 p-2 rounded-full shadow-lg bg-slate-800 text-white hover:bg-slate-700 active:scale-90 z-20 border border-white/10"
            >
              <Sun v-if="isDarkMode" :size="14" class="text-yellow-400" />
              <Moon v-else :size="14" />
            </button>
          </div>
        </template>
      </div>
    </div>
  </div>
</template>

<style>
@import url('https://fonts.googleapis.com/css2?family=Plus+Jakarta+Sans:wght@400;500;600;700;800&display=swap');
/*
[意図] ウィンドウ全体。overscroll-behavior: none を適用し、アプリのスクロールバウンスを抑制。
 CSS の overflow は visible とし、メニュー等のコンテンツがはみ出すことを許可する。 
 s*/
html, body, #app {
  margin: 0;
  padding: 0;
  width: 100%;
  height: 100%;
  background-color: rgba(0, 0, 0, 0) !important;
  overflow: hidden;
  border: none !important;
  outline: none !important;
  box-shadow: none !important;
  overscroll-behavior: none;
}
body { 
  font-family: 'Plus Jakarta Sans', sans-serif; 
}
.custom-scrollbar::-webkit-scrollbar { width: 3px; }
.custom-scrollbar::-webkit-scrollbar-thumb { background: rgba(150, 150, 150, 0.2); border-radius: 10px; }
/* ライトモード用選択色 */
html:not(.dark) ::selection { background: rgba(59, 130, 246, 0.2); color: inherit; }
/* ダークモード用選択色: より明るく白っぽいブルーに調整 */
html.dark ::selection { background: rgba(147, 197, 253, 0.3); color: white; }
</style>
