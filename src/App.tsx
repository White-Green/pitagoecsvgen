import React from 'react';
import './App.css';
import init, { process, natural_sort } from 'pitagoegen_core';

function App() {
  const [files, set_files] = React.useState<TreeNode | undefined>(undefined);
  const [pattern, set_pattern] = React.useState<string>("_${DIR}");
  const [phantom_state, notifier] = React.useState(false);

  return (
    <div className="App">
      <div className="App_left">
        <button type="button" className="form-control header_button" onClick={() => process_input((files) => { set_files(files); set_pattern(files.name + "_${DIR}") })} value="test">フォルダを選択</button>
        <div className="fileviewer_area">
          {files && <TreeViewer node={files} notifier={() => notifier(!phantom_state)} />}
        </div>
      </div>
      <div className="App_right">
        <div className="mb-3">
          <button type="button" className="form-control header_button" onClick={() => generate(files, pattern)} disabled={files == undefined}>生成 &amp; ダウンロード</button>
        </div>
        <div className="mb-3">
          <label className="form-label">
            カテゴリ名生成パターン
            <input type="text" className="form-control cat_pattern_input" value={pattern} onChange={(e) => set_pattern(e.target.value)} disabled={files == undefined}></input>
          </label>
        </div>
      </div>
    </div>
  );
}

function generate(node: TreeNode | undefined, pattern: string) {
  if (node !== undefined) {
    try {
      const all_path = collect_all_files(node);
      console.log(JSON.stringify(all_path));
      const result: string[][] = process(all_path, pattern);
      const result_string = result.map(column => column.map(item => JSON.stringify(item)).join(",")).join("\n");
      let a = document.createElement("a");
      a.href = URL.createObjectURL(new Blob([result_string], { type: "text/csv" }));
      a.download = node.name + ".csv";
      a.click();
    } catch (e) {
      console.error(e);
      alert("エラーが発生しました: ");
    }
  }
}

function collect_all_files(node: TreeNode): string[][] {
  let result: string[][] = [];
  if (node.children !== undefined) {
    result = result.concat(node.children.flatMap(collect_all_files));
  } else if (node.enabled) {
    result.push(node.full_path.slice(1));

  }
  return result;
}

const TreeViewer: React.FC<{ node: TreeNode, visible?: boolean, notifier: () => void }> = ({ node, visible = true, notifier }) => {
  if (node.children === undefined) {
    return (<div className="file_header">
      <label>
        <input type="checkbox" checked={node.enabled} onChange={() => { node.enabled = !node.enabled; notifier(); }} />
        {node.name}
      </label>
    </div>);
  } else {
    const gridTemplateRows = node.children.map(node => `${get_node_size(node, visible) * 40}px`).join(" ");
    return (<div>
      <div className="file_header">
        <label>
          {node.name}
          {node.expanded ? "▼" : "▶"}
          <button className="phantom_button" type="button" onClick={() => { node.expanded = !node.expanded; notifier(); }}>▶</button>
        </label>
      </div>
      <div className="file_children_area">
        {visible && node.expanded && node.children.map(node => <TreeViewer key={node.full_path.join("/")} node={node} visible={visible && node.expanded} notifier={notifier} />)}
      </div>
    </div>);
  }
};

type TreeNode = {
  full_path: string[],
  name: string,
  children?: TreeNode[],
  enabled: boolean,
  expanded: boolean,
};

function get_node_size(node: TreeNode, visible: boolean): number {
  if (node.children === undefined) {
    return visible ? 1 : 0;
  } else {
    return node.expanded && visible ? Math.max(1, node.children.map(node => get_node_size(node, visible && node.expanded)).reduce((a, b) => a + b)) : 1;
  }
}

function process_input(set_files: (node: TreeNode) => void) {
  // @ts-ignore
  Promise.all([init(), window.showDirectoryPicker()])
    .then(async ([_, dir]: [any, FileSystemDirectoryHandle]) => {
      let node = await collect_all_node(dir);
      node.expanded = true;
      set_files(node);
    })
}

async function collect_all_node(dir: FileSystemDirectoryHandle): Promise<TreeNode> {
  return await collect_all_node_inner(dir, []);
}

async function collect_all_node_inner(dir: FileSystemDirectoryHandle, current: string[]): Promise<TreeNode> {
  current.push(dir.name);
  // @ts-ignore
  let entries: any = await dir.entries();
  let children: TreeNode[] = [];
  while (true) {
    let entry = await entries.next();
    if (entry.done === true) break;
    if (entry.value[1].kind === "file") {
      children.push({
        full_path: [...current, entry.value[0]],
        name: entry.value[0],
        children: undefined,
        enabled: true,
        expanded: true,
      });
    } else if (entry.value[1].kind === "directory") {
      const child = await collect_all_node_inner(entry.value[1], current);
      children.push(child);
    }
  }
  const full_path = [...current];
  current.pop();
  const indices = natural_sort(children.map(({ name }) => name));
  let new_children: TreeNode[] = new Array(children.length).fill(undefined);
  for (let i = 0; i < children.length; i++) {
    new_children[i] = children[indices[i]];
  }
  children = new_children;
  children.sort(({ children: a }, { children: b }) => (b === undefined ? 1 : 0) - (a === undefined ? 1 : 0));
  return {
    full_path,
    name: dir.name,
    children,
    enabled: true,
    expanded: false,
  };
}

export default App;
