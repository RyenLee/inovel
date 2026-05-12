#!/usr/bin/env node
/**
 * iNovel Version Updater
 * 一键更新项目中的所有版本号
 * 
 * 使用方式:
 *   node scripts/update-version.cjs <new-version>
 *   例如: node scripts/update-version.cjs 1.1.0
 * 
 * 更新的文件:
 *   - src-tauri/Cargo.toml
 *   - package.json
 *   - src-tauri/tauri.conf.json
 */

const fs = require('fs');
const path = require('path');

const filesToUpdate = [
  {
    path: 'src-tauri/Cargo.toml',
    pattern: /^version = "([^"]+)"/m,
    replace: 'version = "$NEW_VERSION"'
  },
  {
    path: 'package.json',
    pattern: /"version":\s*"([^"]+)"/,
    replace: '"version": "$NEW_VERSION"'
  },
  {
    path: 'src-tauri/tauri.conf.json',
    pattern: /"version":\s*"([^"]+)"/,
    replace: '"version": "$NEW_VERSION"'
  }
];

function updateVersion(fileInfo, newVersion) {
  const fullPath = path.join(__dirname, '..', fileInfo.path);
  
  if (!fs.existsSync(fullPath)) {
    console.log(`❌ 文件不存在: ${fileInfo.path}`);
    return false;
  }

  const content = fs.readFileSync(fullPath, 'utf-8');
  const oldVersionMatch = content.match(fileInfo.pattern);
  
  if (!oldVersionMatch) {
    console.log(`❌ 未找到版本号: ${fileInfo.path}`);
    return false;
  }

  const oldVersion = oldVersionMatch[1];
  
  if (oldVersion === newVersion) {
    console.log(`⚠️ 版本号已相同: ${fileInfo.path} (${oldVersion})`);
    return true;
  }

  const newContent = content.replace(fileInfo.pattern, fileInfo.replace.replace('$NEW_VERSION', newVersion));
  
  fs.writeFileSync(fullPath, newContent, 'utf-8');
  console.log(`✅ ${fileInfo.path}`);
  console.log(`   ${oldVersion} → ${newVersion}`);
  
  return true;
}

function main() {
  const newVersion = process.argv[2];
  
  if (!newVersion) {
    console.error('用法: node scripts/update-version.cjs <new-version>');
    console.error('例如: node scripts/update-version.cjs 1.1.0');
    process.exit(1);
  }

  // 简单验证版本号格式
  const versionRegex = /^\d+\.\d+\.\d+(-[a-zA-Z0-9.]+)?$/;
  if (!versionRegex.test(newVersion)) {
    console.error(`❌ 无效的版本号格式: ${newVersion}`);
    console.error('   请使用语义化版本号格式，如: 1.0.0, 1.1.0, 2.0.0-beta.1');
    process.exit(1);
  }

  console.log(`📦 正在更新版本号: ${newVersion}\n`);
  
  let successCount = 0;
  let failCount = 0;
  
  filesToUpdate.forEach(fileInfo => {
    const result = updateVersion(fileInfo, newVersion);
    if (result) {
      successCount++;
    } else {
      failCount++;
    }
  });

  console.log('\n' + '='.repeat(40));
  
  if (failCount === 0) {
    console.log(`🎉 成功更新 ${successCount} 个文件!`);
    process.exit(0);
  } else {
    console.log(`❌ 更新完成，但有 ${failCount} 个文件失败`);
    process.exit(1);
  }
}

main();