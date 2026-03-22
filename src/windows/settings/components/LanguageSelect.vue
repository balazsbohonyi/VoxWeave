<script setup lang="ts">
import { ref, computed, watch, nextTick } from "vue";

interface Language {
  code: string;
  name: string;
}

const LANGUAGES: Language[] = [
  { code: "af", name: "Afrikaans" },
  { code: "sq", name: "Albanian" },
  { code: "am", name: "Amharic" },
  { code: "ar", name: "Arabic" },
  { code: "hy", name: "Armenian" },
  { code: "as", name: "Assamese" },
  { code: "az", name: "Azerbaijani" },
  { code: "ba", name: "Bashkir" },
  { code: "eu", name: "Basque" },
  { code: "be", name: "Belarusian" },
  { code: "bn", name: "Bengali" },
  { code: "bs", name: "Bosnian" },
  { code: "br", name: "Breton" },
  { code: "bg", name: "Bulgarian" },
  { code: "my", name: "Burmese" },
  { code: "ca", name: "Catalan" },
  { code: "zh", name: "Chinese" },
  { code: "hr", name: "Croatian" },
  { code: "cs", name: "Czech" },
  { code: "da", name: "Danish" },
  { code: "nl", name: "Dutch" },
  { code: "en", name: "English" },
  { code: "et", name: "Estonian" },
  { code: "fo", name: "Faroese" },
  { code: "fi", name: "Finnish" },
  { code: "fr", name: "French" },
  { code: "gl", name: "Galician" },
  { code: "ka", name: "Georgian" },
  { code: "de", name: "German" },
  { code: "el", name: "Greek" },
  { code: "gu", name: "Gujarati" },
  { code: "ht", name: "Haitian Creole" },
  { code: "ha", name: "Hausa" },
  { code: "haw", name: "Hawaiian" },
  { code: "he", name: "Hebrew" },
  { code: "hi", name: "Hindi" },
  { code: "hu", name: "Hungarian" },
  { code: "is", name: "Icelandic" },
  { code: "id", name: "Indonesian" },
  { code: "it", name: "Italian" },
  { code: "ja", name: "Japanese" },
  { code: "jw", name: "Javanese" },
  { code: "kn", name: "Kannada" },
  { code: "kk", name: "Kazakh" },
  { code: "km", name: "Khmer" },
  { code: "ko", name: "Korean" },
  { code: "lo", name: "Lao" },
  { code: "la", name: "Latin" },
  { code: "lv", name: "Latvian" },
  { code: "ln", name: "Lingala" },
  { code: "lt", name: "Lithuanian" },
  { code: "lb", name: "Luxembourgish" },
  { code: "mk", name: "Macedonian" },
  { code: "mg", name: "Malagasy" },
  { code: "ms", name: "Malay" },
  { code: "ml", name: "Malayalam" },
  { code: "mt", name: "Maltese" },
  { code: "mi", name: "Maori" },
  { code: "mr", name: "Marathi" },
  { code: "mn", name: "Mongolian" },
  { code: "ne", name: "Nepali" },
  { code: "no", name: "Norwegian" },
  { code: "nn", name: "Norwegian Nynorsk" },
  { code: "oc", name: "Occitan" },
  { code: "ps", name: "Pashto" },
  { code: "fa", name: "Persian" },
  { code: "pl", name: "Polish" },
  { code: "pt", name: "Portuguese" },
  { code: "pa", name: "Punjabi" },
  { code: "ro", name: "Romanian" },
  { code: "ru", name: "Russian" },
  { code: "sa", name: "Sanskrit" },
  { code: "sr", name: "Serbian" },
  { code: "sn", name: "Shona" },
  { code: "sd", name: "Sindhi" },
  { code: "si", name: "Sinhala" },
  { code: "sk", name: "Slovak" },
  { code: "sl", name: "Slovenian" },
  { code: "so", name: "Somali" },
  { code: "es", name: "Spanish" },
  { code: "su", name: "Sundanese" },
  { code: "sw", name: "Swahili" },
  { code: "sv", name: "Swedish" },
  { code: "tl", name: "Tagalog" },
  { code: "tg", name: "Tajik" },
  { code: "ta", name: "Tamil" },
  { code: "tt", name: "Tatar" },
  { code: "te", name: "Telugu" },
  { code: "th", name: "Thai" },
  { code: "bo", name: "Tibetan" },
  { code: "tr", name: "Turkish" },
  { code: "tk", name: "Turkmen" },
  { code: "uk", name: "Ukrainian" },
  { code: "ur", name: "Urdu" },
  { code: "uz", name: "Uzbek" },
  { code: "vi", name: "Vietnamese" },
  { code: "cy", name: "Welsh" },
  { code: "yi", name: "Yiddish" },
  { code: "yo", name: "Yoruba" },
];

const props = defineProps<{
  modelValue: string;
  disabled?: boolean;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: string];
  change: [value: string];
}>();

const inputRef = ref<HTMLInputElement | null>(null);
const listRef = ref<HTMLUListElement | null>(null);
const query = ref("");
const open = ref(false);
const dropUp = ref(false);
const activeIndex = ref(-1); // -1 = Auto-detect row

function nameForCode(code: string): string {
  return LANGUAGES.find((l) => l.code === code)?.name ?? code;
}

watch(
  () => props.modelValue,
  (val) => {
    if (!open.value) {
      query.value = val ? nameForCode(val) : "";
    }
  },
  { immediate: true },
);

const filtered = computed((): Language[] => {
  const q = query.value.trim().toLowerCase();
  if (!q) return LANGUAGES;
  return LANGUAGES.filter(
    (l) => l.name.toLowerCase().includes(q) || l.code.toLowerCase() === q,
  );
});

function openDropdown() {
  if (props.disabled) return;
  if (inputRef.value) {
    const rect = inputRef.value.getBoundingClientRect();
    dropUp.value = window.innerHeight - rect.bottom < 200;
  }
  query.value = "";
  open.value = true;
  activeIndex.value = -1;
}

function select(code: string) {
  emit("update:modelValue", code);
  emit("change", code);
  query.value = code ? nameForCode(code) : "";
  open.value = false;
  activeIndex.value = -1;
}

function scrollActive() {
  nextTick(() => {
    const el = listRef.value?.querySelector("[data-active='true']") as HTMLElement | null;
    el?.scrollIntoView({ block: "nearest" });
  });
}

function onKeydown(e: KeyboardEvent) {
  if (!open.value) {
    if (e.key === "ArrowDown" || e.key === "Enter") {
      e.preventDefault();
      openDropdown();
    }
    return;
  }
  if (e.key === "Escape") {
    e.preventDefault();
    open.value = false;
    query.value = props.modelValue ? nameForCode(props.modelValue) : "";
    return;
  }
  if (e.key === "ArrowDown") {
    e.preventDefault();
    activeIndex.value = Math.min(activeIndex.value + 1, filtered.value.length - 1);
    scrollActive();
    return;
  }
  if (e.key === "ArrowUp") {
    e.preventDefault();
    activeIndex.value = Math.max(activeIndex.value - 1, -1);
    scrollActive();
    return;
  }
  if (e.key === "Enter") {
    e.preventDefault();
    if (activeIndex.value === -1) {
      select("");
    } else {
      const lang = filtered.value[activeIndex.value];
      if (lang) select(lang.code);
    }
  }
}

function onBlur() {
  setTimeout(() => {
    open.value = false;
    query.value = props.modelValue ? nameForCode(props.modelValue) : "";
  }, 150);
}
</script>

<template>
  <div class="relative">
    <input
      ref="inputRef"
      type="text"
      :value="query"
      :disabled="disabled"
      :placeholder="modelValue ? nameForCode(modelValue) : 'Auto-detect'"
      class="w-40 rounded-md border border-gray-300 bg-white px-2 py-1 text-xs text-gray-900 shadow-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-900 dark:text-gray-100 dark:focus:border-blue-400 dark:focus:ring-blue-400 disabled:cursor-not-allowed disabled:opacity-50"
      @input="query = ($event.target as HTMLInputElement).value; open = true; activeIndex = query.trim() && filtered.length > 0 ? 0 : -1"
      @focus="openDropdown"
      @blur="onBlur"
      @keydown="onKeydown"
    >
    <ul
      v-if="open"
      ref="listRef"
      class="absolute right-0 z-50 max-h-48 w-48 overflow-y-auto rounded-md border border-gray-200 bg-white shadow-lg dark:border-gray-700 dark:bg-gray-900"
      :class="dropUp ? 'bottom-full mb-1' : 'top-full mt-1'"
    >
      <li
        :data-active="activeIndex === -1"
        class="cursor-pointer px-3 py-1.5 text-xs italic"
        :class="activeIndex === -1 ? 'bg-blue-600 text-white' : 'text-gray-400 hover:bg-gray-100 dark:text-gray-500 dark:hover:bg-gray-800'"
        @mousedown.prevent="select('')"
        @mouseover="activeIndex = -1"
      >
        Auto-detect
      </li>
      <li
        v-for="(lang, i) in filtered"
        :key="lang.code"
        :data-active="activeIndex === i"
        class="flex cursor-pointer items-center justify-between px-3 py-1.5 text-xs"
        :class="activeIndex === i ? 'bg-blue-600 text-white' : 'text-gray-900 hover:bg-gray-100 dark:text-gray-100 dark:hover:bg-gray-800'"
        @mousedown.prevent="select(lang.code)"
        @mouseover="activeIndex = i"
      >
        <span>{{ lang.name }}</span>
        <span class="ml-2 opacity-50">{{ lang.code }}</span>
      </li>
    </ul>
  </div>
</template>
