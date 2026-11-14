<template>
  <div class="lock-screen v-theme--light">
    <v-card class="lock-card" elevation="12">
      <div class="lock-icon">
        <img src="/collectrix-logo-flat.svg" alt="Collectrix" class="lock-logo" />
      </div>
      <div class="text-h5 font-weight-bold mt-3">Collectrix</div>
      <div class="text-body-2 text-medium-emphasis mt-1">
        {{ t("输入数字密钥解锁") }}
      </div>

      <v-text-field
        ref="pinInput"
        v-model="pin"
        type="password"
        inputmode="numeric"
        maxlength="12"
        :label="t('数字密钥')"
        density="compact"
        variant="outlined"
        class="mt-5"
        autofocus
        hide-details
        @keydown.enter.prevent="unlock"
      />

      <div v-if="error" class="text-error text-caption mt-2">{{ error }}</div>

      <div class="keypad mt-4">
        <v-btn
          v-for="key in ['1', '2', '3', '4', '5', '6', '7', '8', '9']"
          :key="key"
          variant="tonal"
          class="key"
          @click="append(key)"
        >
          {{ key }}
        </v-btn>
        <v-btn variant="text" class="key" @click="clear">
          {{ t("清空") }}
        </v-btn>
        <v-btn variant="tonal" class="key" @click="append('0')">0</v-btn>
        <v-btn variant="text" class="key" @click="backspace">
          <v-icon>mdi-backspace-outline</v-icon>
        </v-btn>
      </div>

      <v-btn
        block
        color="primary"
        size="large"
        class="mt-4"
        :loading="unlocking"
        :disabled="!pin"
        @click="unlock"
      >
        {{ t("解锁") }}
      </v-btn>
    </v-card>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { t } from "../i18n";

const emit = defineEmits<{ unlocked: [] }>();

const pin = ref("");
const unlocking = ref(false);
const error = ref("");

function append(value: string) {
  if (pin.value.length < 12) pin.value += value;
}

function backspace() {
  pin.value = pin.value.slice(0, -1);
}

function clear() {
  pin.value = "";
  error.value = "";
}

async function unlock() {
  if (!pin.value || unlocking.value) return;
  unlocking.value = true;
  error.value = "";
  try {
    const ok = await invoke<boolean>("verify_app_lock", { pin: pin.value });
    if (ok) {
      emit("unlocked");
    } else {
      error.value = t("密钥错误");
      pin.value = "";
    }
  } catch (e) {
    error.value = String(e);
  } finally {
    unlocking.value = false;
  }
}
</script>

<style scoped>
.lock-screen {
  position: fixed;
  inset: 0;
  z-index: 99999;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #101827, #1b2a44 55%, #223a5e);
  padding: 24px;
}

.lock-card {
  width: min(360px, 100%);
  padding: 28px 24px;
  border-radius: 18px;
  text-align: center;
  background: #ffffff;
  color: #1b1b1f;
}

.lock-logo {
  width: 58px;
  height: 58px;
  border-radius: 14px;
  display: block;
}
.lock-icon {
  width: 76px;
  height: 76px;
  margin: 0 auto;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(25, 118, 210, 0.12);
  color: #1976d2;
}

.keypad {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 8px;
}

.key {
  min-height: 44px;
  font-size: 16px;
}
</style>
