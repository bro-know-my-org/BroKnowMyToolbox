<template>
  <ToolPage :title="t('tools.clipboard.title')" :description="t('tools.clipboard.description')" eyebrow="Built-in Tool">
    <template #actions>
      <NButton @click="readClipboardText">{{ t("tools.clipboard.readText") }}</NButton>
      <NButton @click="writeClipboardText">{{ t("tools.clipboard.writeText") }}</NButton>
      <NButton type="primary" secondary @click="readClipboardImage">{{ t("tools.clipboard.readImage") }}</NButton>
    </template>

    <ToolPanel :title="t('tools.clipboard.panelTitle')" :description="isImage ? t('tools.clipboard.imagePanelDescription') : t('tools.clipboard.textPanelDescription')">
      <div v-if="isImage" class="image-result">
        <div class="meta-grid">
          <div>
            <span>{{ t("tools.clipboard.imageSize") }}</span>
            <strong>{{ imageSize }}</strong>
          </div>
          <div>
            <span>{{ t("tools.clipboard.source") }}</span>
            <strong>{{ t("tools.clipboard.systemClipboard") }}</strong>
          </div>
        </div>
        <img :src="imageUrl" :alt="t('tools.clipboard.imageAlt')" />
      </div>

      <div v-else class="text-result">
        <NInput :value="clipboardText" type="textarea" :rows="12" readonly :placeholder="t('tools.clipboard.textPlaceholder')" />
      </div>
    </ToolPanel>
  </ToolPage>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { NButton, NInput, useMessage } from "naive-ui";
import { readImage, readText, writeText } from "@tauri-apps/plugin-clipboard-manager";
import type { Image } from "@tauri-apps/api/image";
import { useAppI18n } from "../../../i18n/useAppI18n";
import ToolPage from "../../../components/app/ToolPage.vue";
import ToolPanel from "../../../components/app/ToolPanel.vue";

const { t } = useAppI18n();
const message = useMessage();

const clipboardText = ref("");
const imageUrl = ref("");
const imageSize = ref("");
const isImage = ref(false);

type ClipboardImageData = {
  rgba: Uint8Array;
  width: number;
  height: number;
};

async function readClipboardText() {
  try {
    const text = await readText();

    clipboardText.value = text || "";
    isImage.value = false;
  } catch (error) {
    message.error(t("tools.clipboard.readTextFailed", { message: String(error) }));
  }
}

async function writeClipboardText() {
  try {
    await writeText("测试");
    message.success(t("tools.clipboard.writeSuccess"));
  } catch (error) {
    message.error(t("tools.clipboard.writeTextFailed", { message: String(error) }));
  }
}

function toArrayBuffer(view: Uint8Array): ArrayBuffer {
  return view.buffer.slice(view.byteOffset, view.byteOffset + view.byteLength) as ArrayBuffer;
}

function createClipboardImageData(rgba: Uint8Array, width: number, height: number): ImageData {
  return new ImageData(new Uint8ClampedArray(toArrayBuffer(rgba)), width, height);
}

function imageDataToDataUrl(imageData: ImageData): string {
  const canvas = document.createElement("canvas");
  const context = canvas.getContext("2d");

  if (!context) {
    throw new Error("无法创建 Canvas 上下文");
  }

  canvas.width = imageData.width;
  canvas.height = imageData.height;
  context.putImageData(imageData, 0, 0);

  return canvas.toDataURL("image/png");
}

async function readClipboardImageData(clipboardImage: Image): Promise<ClipboardImageData> {
  const rgba = await clipboardImage.rgba();
  const { width, height } = await clipboardImage.size();

  return { rgba, width, height };
}

async function readClipboardImage() {
  try {
    const clipboardImage = await readImage();
    const { rgba, width, height } = await readClipboardImageData(clipboardImage);
    const imageData = createClipboardImageData(rgba, width, height);

    imageUrl.value = imageDataToDataUrl(imageData);
    imageSize.value = `${width} x ${height}`;
    isImage.value = true;
  } catch (error) {
    isImage.value = false;
    message.error(t("tools.clipboard.readImageFailed", { message: String(error) }));
  }
}
</script>

<style scoped lang="scss">
.text-result {
  max-width: 980px;
}

.image-result {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.meta-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(160px, 1fr));
  gap: 12px;
}

.meta-grid div {
  border: 1px solid var(--bkm-border);
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.025);
  padding: 12px;
}

.meta-grid span {
  display: block;
  color: var(--bkm-text-muted);
  font-size: 12px;
}

.meta-grid strong {
  display: block;
  margin-top: 4px;
  color: var(--bkm-text);
  font-size: 15px;
}

img {
  max-width: min(100%, 980px);
  border: 1px solid var(--bkm-border);
  border-radius: 8px;
  background: #0f151b;
}
</style>
