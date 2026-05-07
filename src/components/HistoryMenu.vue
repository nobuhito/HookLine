<script setup>
import { X } from 'lucide-vue-next';

const props = defineProps({
  isOpen: Boolean,
  history: Array,
  styles: Object
});

const emit = defineEmits(['close', 'copy']);
</script>

<template>
  <div v-if="isOpen" :class="[styles.dropdown, 'right-0 top-0 origin-top-right']">
    <div :class="['flex justify-between items-center px-5 py-3.5 border-b shrink-0', styles.borderBase]">
      <span class="text-[9px] font-black text-slate-400 uppercase tracking-widest">Recent Activity</span>
      <X :size="16" class="text-slate-400 cursor-pointer hover:text-slate-600" @click="emit('close')" />
    </div>
    <div class="flex-1 overflow-y-auto p-2 custom-scrollbar">
      <template v-if="history.length > 0">
        <div v-for="item in history" :key="item.id" @click="emit('copy', item.text)" 
          :class="['p-3 rounded-[12px] transition-all group cursor-pointer mb-1', styles.itemHover]">
          <div class="text-[12px] leading-relaxed font-medium mb-1 line-clamp-2">{{ item.text }}</div>
          <div class="flex justify-between items-center opacity-40">
            <span class="text-[8px] font-black uppercase tracking-wider">{{ item.profile }}</span>
            <span class="text-[8px] font-black uppercase tracking-wider">
              {{ 
                typeof item.timestamp === 'number' 
                  ? new Date(item.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
                  : item.timestamp 
              }}
            </span>
          </div>
        </div>
      </template>
      <div v-else class="p-10 text-center text-slate-400 text-[10px] font-bold uppercase tracking-widest">
        No History
      </div>
    </div>
  </div>
</template>
