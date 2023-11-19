<template>
  <div>
    <el-input v-model="selected" placeholder="请选择文件夹" @click="selectDir"></el-input>
    <el-button type="primary" @click=getDatas v-loading.fullscreen.lock="searchLoading"
      :element-loading-text="`已经获取 ${count} 个文件`">查询文件</el-button>
    <el-button type="primary" @click=getSingleDirDatas v-loading.fullscreen.lock="searchLoading"
      :element-loading-text="`已经获取 ${count} 个文件`">查询单文件重复文件</el-button>
    <el-button type="primary" @click=caculateSha1>计算sha1</el-button>
    <el-button type="danger" @click="openDialog">删除文件</el-button>
    <el-progress :percentage="progressPercent" text-inside :stroke-width="25" />
  </div>
  <div style="height: calc(100% - 89px);;display: flex;">
    <el-auto-resizer style="width: 50%;">
      <template #default="{ height, width }">
        <el-table-v2 :columns="leftColumns" :data="leftTableData" :height="height" :width="width" expand-column-key="len"
          :row-class="leftRowClass" :default-expanded-row-keys="defaultExpandedRowKeys" v-loading.lock="tableLoading">
          <template #row="props">
            <LiftRow v-bind="props" />
          </template>
        </el-table-v2>
      </template>
    </el-auto-resizer>
    <el-auto-resizer style="width: 50%">
      <template #default="{ height, width }">
        <el-table-v2 :columns="rightColumns" :data="rightTableData" :height="height" :width="width"
          expand-column-key="len" :row-class="rowClass" :default-expanded-row-keys="defaultExpandedRowKeys"
          v-loading.lock="tableLoading">
          <template #row="props">
            <RightRow v-bind="props" />
          </template>
        </el-table-v2>
      </template>
    </el-auto-resizer>

  </div>
  <el-dialog v-model="dialogVisible" title="删除文件" @keyup.enter="deleteFiles">
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
import { Ref, cloneVNode, ref } from 'vue';
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
const leftColumns: any = [
  { key: "id", dataKey: "id", title: "序号", width: 100, align: "center" },
  { key: "sha1", dataKey: "sha1", title: "SHA1", width: 400, align: "center" },
  { key: "modified_time", dataKey: "modified_time", title: "修改日期", width: 200, align: "center" },
  { key: "created_time", dataKey: "created_time", title: "创建日期", width: 200, align: "center" },
  { key: "filename", dataKey: "filename", title: "路径", width: 400, align: "right" },
  { key: "len", dataKey: "len", title: "文件大小", width: 100, align: "right" },
  {
    key: "select", width: 50, align: "center",
    cellRenderer: ({ rowData }: any) => {
      const onChange = (value: CheckboxValueType) => (rowData.checked = value)
      return <ElCheckbox modelValue={rowData.checked} onChange={onChange} />
    },
    headerCellRenderer: () => {
      const _data: any = leftTableData.value
      const onChange = (value: CheckboxValueType) =>
      (leftTableData.value = _data.map((row: any) => {
        row.checked = value
        return row
      }))
      const allSelected = _data.every((row: any) => row.checked)
      const containsChecked = _data.some((row: any) => row.checked)
      return <ElCheckbox modelValue={allSelected} indeterminate={containsChecked && !allSelected} onChange={onChange} />
    }
  },
]
const rightColumns: any = [
  {
    key: "select", width: 50, align: "center",
    cellRenderer: ({ rowData }: any) => {
      const onChange = (value: CheckboxValueType) => (rowData.checked = value)
      return <ElCheckbox modelValue={rowData.checked} onChange={onChange} />
    },
    headerCellRenderer: () => {
      const _data: any = rightTableData.value
      const onChange = (value: CheckboxValueType) =>
      (rightTableData.value = _data.map((row: any) => {
        row.checked = value
        return row
      }))
      const allSelected = _data.every((row: any) => row.checked)
      const containsChecked = _data.some((row: any) => row.checked)
      return <ElCheckbox modelValue={allSelected} indeterminate={containsChecked && !allSelected} onChange={onChange} />
    }
  },
  { key: "len", dataKey: "len", title: "文件大小", width: 100 },
  { key: "filename", dataKey: "filename", title: "路径", width: 400 },
  { key: "created_time", dataKey: "created_time", title: "创建日期", width: 200, align: "center" },
  { key: "modified_time", dataKey: "modified_time", title: "修改日期", width: 200, align: "center" },
  { key: "sha1", dataKey: "sha1", title: "SHA1", width: 400, align: "center" },
  { key: "id", dataKey: "id", title: "序号", width: 100, align: "center" }
]
const leftTableData: Ref<any[]> = ref([])
const rightTableData: Ref<any[]> = ref([])
let repeatedCount: any = {}

let searchLoading = ref(false)
let count = 0
const getSingleDirDatas = async () => {
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

  leftTableData.value = []
  rightTableData.value = []
  dirPair = []
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
        let cur = fileInfo.path.split('\\')
        cur.pop()
        if (JSON.stringify(cur) == JSON.stringify(pre)) {
          fileInfo.checked = true
        }
        repeatedCount[fileInfo.sha1]++
      }
    }
  }

  for (let i = 0; i < resDatas.length; i++) {
    const curFileInfo = resDatas[i];
    if (curFileInfo.path.search(/U:\\阿里云盘\\聊天记录\\MsgBackup\\.*?-journal/) == 0) {
      continue
    }
    if (repeatedCount[curFileInfo.sha1] > 1) {
      curFileInfo.created_time = dayjs(parseInt(curFileInfo.created_time)).format("YYYY-MM-DD HH:MM:ss")
      curFileInfo.modified_time = dayjs(parseInt(curFileInfo.modified_time)).format("YYYY-MM-DD HH:MM:ss")

      curFileInfo.filename = curFileInfo.path.split('\\').pop()
      leftTableData.value.push(curFileInfo)
    }
  }

  clearInterval(timer)
  searchLoading.value = false
}
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

  leftTableData.value = []
  rightTableData.value = []
  dirPair = []
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
        let cur = fileInfo.path.split('\\')
        cur.pop()
        if (JSON.stringify(cur) == JSON.stringify(pre)) {
          fileInfo.checked = true
        }
        repeatedCount[fileInfo.sha1]++
      }
    }
  }

  let sha1
  for (let i = 0; i < resDatas.length; i++) {
    const curFileInfo = resDatas[i];
    if (curFileInfo.path.search(/U:\\阿里云盘\\聊天记录\\MsgBackup\\.*?-journal/) == 0) {
      continue
    }
    if (curFileInfo.sha1 == sha1) {
      continue
    }
    if (repeatedCount[curFileInfo.sha1] % 2 == 0) {
      sha1 = curFileInfo.sha1
      const nexFileInfo = resDatas[i + 1]
      curFileInfo.created_time = dayjs(parseInt(curFileInfo.created_time)).format("YYYY-MM-DD HH:MM:ss")
      curFileInfo.modified_time = dayjs(parseInt(curFileInfo.modified_time)).format("YYYY-MM-DD HH:MM:ss")
      nexFileInfo.created_time = dayjs(parseInt(nexFileInfo.created_time)).format("YYYY-MM-DD HH:MM:ss")
      nexFileInfo.modified_time = dayjs(parseInt(nexFileInfo.modified_time)).format("YYYY-MM-DD HH:MM:ss")

      new FileInfo(curFileInfo, nexFileInfo).walk()
    }
  }

  clearInterval(timer)
  searchLoading.value = false
}
const leftRowClass = ({ rowData }: any) => {
  if (repeatedCount[rowData.sha1] > 1) {
    return 'leftIsRepeated'
  }
  if (rowData.sha1 == undefined) {
    return 'isDir'
  }
}
const rowClass = ({ rowData }: any) => {
  if (repeatedCount[rowData.sha1] > 1) {
    return 'isRepeated'
  }
  if (rowData.sha1 == undefined) {
    return 'isDir'
  }
}

const leftcolSpanIndex = 5
const LiftRow = ({ rowData, rowIndex, cells, columns }: any) => {
  if (rowData.sha1 == undefined) {
    const colSpan = 6
    let width = Number.parseInt(cells[leftcolSpanIndex].props.style.width)
    for (let i = 1; i < colSpan; i++) {
      width += Number.parseInt(cells[leftcolSpanIndex - i].props.style.width)
      cells[leftcolSpanIndex - i] = null
    }
    const style = {
      ...cells[leftcolSpanIndex].props.style,
      width: `${width}px`,
    }
    cells[leftcolSpanIndex] = cloneVNode(cells[leftcolSpanIndex], { style })
  }
  return cells
}
const rightcolSpanIndex = 1
const RightRow = ({ rowData, rowIndex, cells, columns }: any) => {
  if (rowData.sha1 == undefined) {
    const colSpan = 6
    let width = Number.parseInt(cells[rightcolSpanIndex].props.style.width)
    for (let i = 1; i < colSpan; i++) {
      width += Number.parseInt(cells[rightcolSpanIndex + i].props.style.width)
      cells[rightcolSpanIndex + i] = null
    }
    const style = {
      ...cells[rightcolSpanIndex].props.style,
      width: `${width}px`,
    }
    cells[rightcolSpanIndex] = cloneVNode(cells[rightcolSpanIndex], { style })
  }
  return cells
}
const defaultExpandedRowKeys: Ref<any[]> = ref([])

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
  getCheckedFilePath(leftTableData.value)
  getCheckedFilePath(rightTableData.value)
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

let dirPair: any[] = []
class FileInfo {
  leftFile
  rightFile

  constructor(preFileInfo: any, nexFileInfo: any) {
    this.leftFile = {
      id: preFileInfo.id,
      len: preFileInfo.len,
      created_time: preFileInfo.created_time,
      modified_time: preFileInfo.modified_time,
      path: preFileInfo.path,
      sha1: preFileInfo.sha1,
      checked: preFileInfo.checked,
    }
    this.rightFile = {
      id: nexFileInfo.id,
      len: nexFileInfo.len,
      created_time: nexFileInfo.created_time,
      modified_time: nexFileInfo.modified_time,
      path: nexFileInfo.path,
      sha1: nexFileInfo.sha1,
      checked: nexFileInfo.checked,
    }
  }
  walk() {
    let pathBuf = this.leftFile.path.split('\\')
    let filename = pathBuf.pop()
    let leftdir = pathBuf.join('\\')

    let rightpathBuf = this.rightFile.path.split('\\')
    let rightfilename = rightpathBuf.pop()
    let rightdir = rightpathBuf.join('\\')

    //如果文件夹不在默认展开，则添入
    if (!defaultExpandedRowKeys.value.find((x: any) => x == leftdir)) {
      defaultExpandedRowKeys.value.push(leftdir)
    }
    //如果文件夹不在默认展开，则添入
    if (!defaultExpandedRowKeys.value.find((x: any) => x == rightdir)) {
      defaultExpandedRowKeys.value.push(rightdir)
    }

    let pairdir = dirPair.find(x => x[leftdir] != undefined && x[rightdir] != undefined)

    if (pairdir == undefined) {
      //如果是文件夹
      let length = leftTableData.value.push({ id: leftdir, len: leftdir, children: [] })
      //如果是文件夹
      let rightlength = rightTableData.value.push({ id: rightdir, len: rightdir, children: [] })
      let obj: any = {}
      obj[leftdir] = length - 1
      obj[rightdir] = rightlength - 1
      dirPair.push(obj)
      pairdir = obj
    }

    let leftindex = pairdir[leftdir]
    let rightindex = pairdir[rightdir]
    leftTableData.value[leftindex].children.push({
      id: this.leftFile.id,
      len: this.leftFile.len,
      created_time: this.leftFile.created_time,
      modified_time: this.leftFile.modified_time,
      path: this.leftFile.path,
      sha1: this.leftFile.sha1,
      checked: this.leftFile.checked,
      filename
    })

    rightTableData.value[rightindex].children.push({
      id: this.rightFile.id,
      len: this.rightFile.len,
      created_time: this.rightFile.created_time,
      modified_time: this.rightFile.modified_time,
      path: this.rightFile.path,
      sha1: this.rightFile.sha1,
      checked: this.rightFile.checked,
      filename: rightfilename
    })
  }
}
</script>

<style scoped>
:global(.el-table-v2 .isRepeated>div:nth-child(6)) {
  background-color: #f56c6c;
}

:global(.el-table-v2 .leftIsRepeated>div:nth-child(2)) {
  background-color: #f56c6c;
}

:global(.el-table-v2 .isDir) {
  background-color: #ffe8a0;
}
</style>