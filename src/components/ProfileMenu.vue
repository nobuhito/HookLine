<script setup>
import { ref, onMounted, onUnmounted, computed, watch } from 'vue';
import { 
  X, ChevronLeft, Settings, Plus, FileText, Trash2, Check 
} from 'lucide-vue-next';

const props = defineProps({
  isOpen: Boolean,
  profiles: Array,
  activeProfileId: Number,
  styles: Object,
  isDarkMode: Boolean,
  onAdd: Function
});

watch(() => props.isOpen, (val) => {
  console.log("ProfileMenu isOpen:", val);
});

const emit = defineEmits(['close', 'select', 'save', 'delete']);

const viewMode = ref('list'); // 'list' | 'edit'
const editingProfile = ref(null);

const APP_CONFIG = {
  ANIMATION_DELAY_MS: 200,
  MAX_PROFILES: 9
};

const openEditMode = (profile) => {
  editingProfile.value = { ...profile };
  viewMode.value = 'edit';
};

const handleAdd = () => {
  if (props.onAdd) {
    const newProfile = props.onAdd();
    if (newProfile) {
      openEditMode(newProfile);
    }
  }
};

// Handle closing and resetting viewMode
onMounted(() => {
  // We can't easily watch 'isOpen' props in a way that triggers a timer on close inside the component 
  // without a watcher, but the React code did:
  // useEffect(() => { if (!isOpen) ... }, [isOpen])
});
</script>

<template>
  <div v-if="isOpen" :class="[styles.dropdown, 'left-0 top-0 origin-top-left']">
    <div :class="['flex items-center justify-between px-5 py-3.5 border-b shrink-0', styles.borderBase]">
      <div class="flex items-center gap-2 cursor-pointer group" @click="viewMode === 'edit' ? viewMode = 'list' : emit('close')">
        <template v-if="viewMode === 'edit'">
          <div class="flex items-center gap-2 text-slate-400 group-hover:text-slate-200">
            <ChevronLeft :size="16" />
            <span class="font-bold text-[10px] uppercase tracking-wider">Back to List</span>
          </div>
        </template>
        <template v-else>
          <span :class="['font-black text-[9px] uppercase tracking-[0.2em]', isDarkMode ? 'text-slate-400' : 'text-slate-500']">Profile</span>
        </template>
      </div>
      <X :size="16" class="text-slate-400 cursor-pointer hover:text-slate-600" @click="emit('close')" />
    </div>

    <div class="flex-1 overflow-y-auto custom-scrollbar">
      <div v-if="viewMode === 'list'" class="p-2">
        <div v-for="p in profiles" :key="p.id" @click="emit('select', p.id)"
          :class="[
            'group flex items-center justify-between p-2.5 rounded-[12px] transition-all cursor-pointer mb-1',
            activeProfileId === p.id ? 'bg-[#3B82F6] text-white shadow-lg' : styles.itemHover
          ]"
        >
          <div class="flex items-center gap-3">
            <span class="w-4 text-[9px] font-bold opacity-40 text-center">{{ p.id }}</span>
            <span class="font-bold text-[12px]">{{ p.name }}</span>
          </div>
          <button @click.stop="openEditMode(p)"
            :class="['p-1.5 rounded-full hover:bg-black/10 transition-all', activeProfileId === p.id ? 'opacity-100' : 'opacity-0 group-hover:opacity-100']">
            <Settings :size="14" />
          </button>
        </div>
        
        <button v-if="profiles.length < APP_CONFIG.MAX_PROFILES" 
          @click="handleAdd" 
          :class="['w-full flex items-center justify-center gap-2 p-2.5 mt-1 border border-dashed rounded-[12px] text-slate-400 hover:text-blue-500 hover:border-blue-500/50 transition-all text-[10px] font-bold uppercase tracking-wider', styles.borderBase]">
          <Plus :size="14" /> Add Profile
        </button>
      </div>

      <div v-else class="p-5 space-y-4">
        <div class="space-y-1">
          <label class="text-[8px] font-black uppercase text-slate-400 px-1 tracking-widest">Name</label>
          <input type="text" v-model="editingProfile.name" :class="styles.inputField" />
        </div>
        <div class="space-y-1">
          <label class="text-[8px] font-black uppercase text-slate-400 px-1 tracking-widest">Target Path</label>
          <div class="relative">
            <FileText :size="12" class="absolute left-3 top-2.5 text-slate-400" />
            <input type="text" v-model="editingProfile.target" :class="[styles.inputField, 'pl-9 font-mono text-[11px]']" />
          </div>
        </div>
        <div class="space-y-1">
          <label class="text-[8px] font-black uppercase text-slate-400 px-1 tracking-widest">Command</label>
          <textarea v-model="editingProfile.command" class="w-full bg-[#020617] text-emerald-400 p-3 rounded-lg font-mono text-[10px] h-24 resize-none outline-none border border-white/5 focus:border-emerald-500/50 transition-colors" />
        </div>
        <div class="flex gap-2 pt-1">
          <button @click="emit('delete', editingProfile.id); viewMode = 'list'" class="flex items-center justify-center w-10 h-10 rounded-lg border border-rose-500/20 text-rose-500 hover:bg-rose-500 hover:text-white transition-all shrink-0">
            <Trash2 :size="16" />
          </button>
          <button @click="emit('save', editingProfile); viewMode = 'list'" :class="['flex-1 text-white rounded-lg text-[10px] font-black tracking-[0.1em] uppercase hover:opacity-90 active:scale-[0.98] transition-all flex items-center justify-center gap-2', isDarkMode ? 'bg-blue-600' : 'bg-[#0F172A]']">
            <Check :size="14" /> Apply
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
