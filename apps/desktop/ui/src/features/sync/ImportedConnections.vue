<script setup lang="ts">
import { onMounted,ref } from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import { errorText } from '@/features/models/api'
import { listImportedConnections,activateImportedConnection,type ImportedConnection } from './api'
const emit=defineEmits<{activated:[]}>()
const items=ref<ImportedConnection[]>([]);const baseline=ref('');const selected=ref<ImportedConnection|null>(null);const busy=ref(false);const error=ref('')
async function load(){if(busy.value)return;busy.value=true;error.value='';try{const result=await listImportedConnections();items.value=result.items;baseline.value=result.baseline}catch(cause){error.value=errorText(cause)}finally{busy.value=false}}
onMounted(()=>{void load()})
async function activate(){if(!selected.value||busy.value)return;busy.value=true;error.value='';try{await activateImportedConnection(selected.value.id,baseline.value);selected.value=null;emit('activated')}catch(cause){error.value=errorText(cause)}finally{busy.value=false}if(!error.value)await load()}
</script>
<template>
 <article v-if="items.length||error" class="card imported-connections"><div class="card-head"><div><h2>导入的连接</h2><p>采用文件不会自动启用连接。启用后，后续同步使用所选服务器。</p></div><UiButton size="sm" :loading="busy" @click="load">刷新</UiButton></div><p v-if="error&&!selected" class="imported-error" role="alert">{{ error }}</p><div v-for="item in items" :key="item.id" class="imported-row"><div><strong>{{ item.endpoint.username }} · {{ item.endpoint.url }}</strong><small>{{ item.endpoint.remotePath }} · {{ item.hasPassword?'已携带密码':'未携带密码，请在连接表单补全' }}</small></div><UiButton :disabled="busy||!item.hasPassword" @click="selected=item">启用此连接</UiButton></div>
  <UiDialog :open="!!selected" title="启用导入的WebDAV连接" :busy="busy" @close="selected=null"><template v-if="selected"><p>后续同步将使用以下服务器；本设备同步范围保持。</p><p>{{ selected.endpoint.url }} / {{ selected.endpoint.remotePath }}</p><p>账号：{{ selected.endpoint.username }}</p></template><p v-if="error" role="alert" class="imported-error">{{ error }}</p><template #actions><UiButton :disabled="busy" @click="selected=null">取消</UiButton><UiButton variant="primary" :loading="busy" @click="activate">确认启用</UiButton></template></UiDialog>
 </article>
</template>
<style scoped>
.imported-row{display:flex;gap:16px;align-items:center;justify-content:space-between;border-top:1px solid var(--border);padding:14px 0}.imported-row>div{min-width:0;overflow-wrap:anywhere}.imported-row small{display:block;color:var(--muted);margin-top:6px}.imported-error{color:var(--red);overflow-wrap:anywhere}
</style>
