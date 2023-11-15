<template>
  <div>
    <el-input v-model="selected" placeholder="请选择文件夹" @click="selectDir"></el-input>
    <el-button type="primary" @click=getDatas v-loading.fullscreen.lock="searchLoading"
      :element-loading-text="`已经获取 ${count} 个文件`">查询文件</el-button>
    <el-button type="primary" @click=caculateSha1>计算sha1</el-button>
    <el-button type="danger" @click="openDialog">删除文件</el-button>
    <el-switch v-model="inOneDir" active-text="同一文件夹" inactive-text="不同文件夹" />
    <el-progress :percentage="progressPercent" text-inside :stroke-width="25" />
  </div>
  <el-auto-resizer style="height: calc(100% - 89px);">
    <template #default="{ height, width }">
      <el-table-v2 :columns="columns" :data="tableData" :height="height" :width="width" expand-column-key="id"
        :row-class="rowClass" :default-expanded-row-keys="defaultExpandedRowKeys" v-loading.lock="tableLoading" />
    </template>
  </el-auto-resizer>
  <el-dialog v-model="dialogVisible" title="删除文件">
    <el-table :data="files">
      <el-table-column label="路径" prop="path"></el-table-column>
    </el-table>
    <template #footer>
      <span class="dialog-footer">
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" @click="deleteFiles">确认</el-button>
      </span>
    </template>
  </el-dialog>
</template>

<script setup lang="tsx">
import { Ref, ref } from 'vue';
import { open } from '@tauri-apps/api/dialog'
import { invoke } from "@tauri-apps/api/tauri";
import { CheckboxValueType, ElCheckbox, dayjs } from 'element-plus';
const selected = ref('')
const selectDir = () => {
  open({
    directory: true
  }).then((r: any) => {
    selected.value = r
  })
}
const columns: any = [
  {
    key: "select", width: 50, align: "center",
    cellRenderer: ({ rowData }: any) => {
      const onChange = (value: CheckboxValueType) => (rowData.checked = value)
      return <ElCheckbox modelValue={rowData.checked} onChange={onChange} />
    },
    headerCellRenderer: () => {
      const _data: any = tableData.value
      const onChange = (value: CheckboxValueType) =>
      (tableData.value = _data.map((row: any) => {
        row.checked = value
        return row
      }))
      const allSelected = _data.every((row: any) => row.checked)
      const containsChecked = _data.some((row: any) => row.checked)
      return <ElCheckbox modelValue={allSelected} indeterminate={containsChecked && !allSelected} onChange={onChange} />
    }
  },
  { key: "id", dataKey: "id", title: "序号", width: 200 },
  { key: "path", dataKey: "path", title: "路径", width: 1000 },
  { key: "len", dataKey: "len", title: "文件大小", width: 1000, align: "center" },
  { key: "modified_time", dataKey: "modified_time", title: "修改日期", width: 200, align: "center" },
  { key: "sha1", dataKey: "sha1", title: "SHA1", width: 400, align: "center" }
]
const tableData: Ref<any[]> = ref([])
let repeatedCount: any = {}

let searchLoading = ref(false)
let count = 0
let inOneDir = ref(true)
const getDatas = async () => {
  if (selected.value == "") {
    return
  }
  searchLoading.value = true
  count = 0

  let loadingTextHtml = await Promise.resolve().then(() => {
    return document.querySelector(".el-loading-text")
  })
  let timer = setInterval(() => {
    invoke('get_walkfile_count').then((res: any) => {
      count = res
      loadingTextHtml!.innerHTML = `已经获取 ${count} 个文件`
    })
  }, 1000)

  const res: string = await invoke("get_datas", { path: selected.value })
  const resDatas = JSON.parse(res)

  tableData.value = []
  repeatedCount = {}
  for (let i = 0; i < resDatas.length; i++) {
    const fileInfo = resDatas[i];
    if (fileInfo.sha1 != null) {
      if (repeatedCount[fileInfo.sha1] == undefined) {
        repeatedCount[fileInfo.sha1] = 1
      } else {
        //已经有相同的sha1，该文件重复
        let pre = resDatas[i - 1].path.split('\\')
        pre.pop()
        let cur = resDatas[i].path.split('\\')
        cur.pop()
        if (JSON.stringify(pre) == JSON.stringify(cur)) {
          resDatas[i].checked = true
        }
        repeatedCount[fileInfo.sha1]++
      }
    }
  }
  if (inOneDir.value) {
    for (let i = 0; i < resDatas.length; i++) {
      const fileInfo = resDatas[i];
      if (fileInfo.path.search(/U:\\阿里云盘\\聊天记录\\MsgBackup/) == 0) {
        continue
      }
      fileInfo.modified_time = dayjs(parseInt(fileInfo.modified_time)).format("YYYY-MM-DD HH:MM:ss")
      new FileInfo(fileInfo).walk()
    }
  } else {
    for (let i = 0; i < resDatas.length; i++) {
      const fileInfo = resDatas[i];
      if (fileInfo.path.search(/U:\\阿里云盘\\聊天记录\\MsgBackup/) == 0) {
        continue
      }
      fileInfo.modified_time = dayjs(parseInt(fileInfo.modified_time)).format("YYYY-MM-DD HH:MM:ss")
      //重复文件
      if (fileInfo.sha1 != null && repeatedCount[fileInfo.sha1] > 1) {
        let dirpath = fileInfo.path.split('\\')
        let filename = dirpath.pop()
        fileInfo.len = dirpath

        if (fileInfo.path.search(/文件恢复/) != -1) {
          fileInfo.checked = true
        }
        tableData.value.push(fileInfo)
      }
    }
  }

  clearInterval(timer)
  searchLoading.value = false
}
const rowClass = ({ rowData }: any) => {
  if (repeatedCount[rowData.sha1] > 1) {
    return 'isRepeated'
  }
  if (rowData.len == undefined) {
    return 'isDir'
  }
}
const defaultExpandedRowKeys: any[] = []

let progressPercent = ref(0)
let tableLoading = ref(false)
const caculateSha1 = async () => {
  tableLoading.value = true
  progressPercent.value = 0
  let timer = setInterval(() => {
    invoke('get_progress_percent').then((res: any) => {
      progressPercent.value = res
    })
  }, 1000)
  await invoke('caculate_sha1', { path: selected.value })
  clearInterval(timer)
  progressPercent.value = 100
  tableLoading.value = false
  getDatas()
}

const dialogVisible = ref(false)
const files: Ref<any[]> = ref([])

const openDialog = () => {
  files.value = []
  const getCheckedFilePath = (tableData: any) => {
    for (let i = 0; i < tableData.length; i++) {
      const fileInfo = tableData[i];
      if (fileInfo.checked) {
        files.value.push({ path: fileInfo.path })
      } else {
        if (fileInfo.children && fileInfo.children.length > 0) {
          getCheckedFilePath(fileInfo.children)
        }
      }
    }
  }
  getCheckedFilePath(tableData.value)
  dialogVisible.value = true
}
const deleteFiles = async () => {
  dialogVisible.value = false
  tableLoading.value = true

  let vecPaths = []
  for (let i = 0; i < files.value.length; i++) {
    const element = files.value[i];
    vecPaths.push(element.path)
  }
  const res = await invoke('delete_files', { vecPaths: vecPaths })
  alert(res)
  tableLoading.value = false
  getDatas()
}

class FileInfo {
  id
  len
  modified_time
  path
  sha1
  checked

  constructor(fileInfo: any) {
    this.id = fileInfo.id
    this.len = fileInfo.len
    this.modified_time = fileInfo.modified_time
    this.path = fileInfo.path
    this.sha1 = fileInfo.sha1
    this.checked = fileInfo.checked
  }
  walk() {
    //不重复文件不显示
    if (repeatedCount[this.sha1] <= 1 || this.sha1 == null) {
      return
    }
    let temp = tableData.value
    let pathBuf = this.path.split('\\')
    for (let i = 0; i < pathBuf.length; i++) {
      const path = pathBuf[i];
      let index = temp.findIndex((x: any) => x.path == path)
      if (index == -1) {
        //如果是文件夹
        if (i != pathBuf.length - 1) {
          temp.push({ id: path, path: path, children: [] })
          //如果文件夹不在默认展开，则添入
          if (!defaultExpandedRowKeys.find((x: any) => x.path == path)) {
            defaultExpandedRowKeys.push(path)
          }
        } else {
          temp.push({ id: this.id, path: this.path, len: this.len, modified_time: this.modified_time, sha1: this.sha1, checked: this.checked, children: [] })
        }
      }
      temp = temp[index == -1 ? temp.length - 1 : index].children
    }
  }
}
</script>

<style scoped>
:global(.el-table-v2 .isRepeated>div:nth-child(6)) {
  background-color: #f56c6c;
}

:global(.el-table-v2 .isDir) {
  background-color: #ffe8a0;
}
</style>