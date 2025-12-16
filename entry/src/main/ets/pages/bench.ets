/*
  * Copyright (c) 2022 Huawei Device Co., Ltd.
  *
  * Licensed under the Apache License, Version 2.0 (the "License");
  * you may not use this file except in compliance with the License.
  * You may obtain a copy of the License at
    *
  * http://www.apache.org/licenses/LICENSE-2.0
    *
  * Unless required by applicable law or agreed to in writing, software
  * distributed under the License is distributed on an "AS IS" BASIS,
  * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
  * See the License for the specific language governing permissions and
  * limitations under the License.
  */
import protobuf from '@ohos/protobufjs';
import { user } from './user.js';
import Long from 'long';

@Entry
@Component
struct Index {
  @State statusText: string = 'Checking...';
  @State implDetails: string = '';
  @State testResult: string = '';

  aboutToAppear() {
    this.checkImplementation();
  }

  checkImplementation() {
    try {
      // ========================================
      // 方法 1: 检查当前实现类型
      // ========================================
      const impl = protobuf.getImplementation();
      console.log('🔍 Current implementation:', impl);

      // ========================================
      // 方法 2: 检查详细信息
      // ========================================
      const details = protobuf.getImplementationDetails();
      console.log('📊 Implementation details:', JSON.stringify(details, null, 2));

      // ========================================
      // 方法 3: 检查 Reader 和 Writer 对象
      // ========================================
      console.log('📖 Reader constructor name:', protobuf.Reader.name);
      console.log('✍️  Writer constructor name:', protobuf. Writer.name);
      console.log('🔧 Reader has _nativeAvailable:', protobuf.Reader._nativeAvailable);
      console.log('🔧 Writer has _nativeAvailable:', protobuf.Writer._nativeAvailable);

      // ========================================
      // 方法 4: 比较对象引用
      // ========================================
      const isRustReader = protobuf.Reader !== protobuf._originalReader;
      const isRustWriter = protobuf.Writer !== protobuf._originalWriter;
      console.log('🆚 Reader is Rust:', isRustReader);
      console.log('🆚 Writer is Rust:', isRustWriter);

      // ========================================
      // 方法 5: 检查是否有 Rust 特有方法
      // ========================================
      const hasRustFlag = protobuf._rustAvailable;
      const hasRustIntegrated = protobuf._rustIntegrated;
      console.log('🦀 Rust available:', hasRustFlag);
      console.log('🔗 Rust integrated:', hasRustIntegrated);

      // 汇总结果
      this. statusText = `当前实现: ${impl. toUpperCase()}\n` +
        `Rust 集成: ${hasRustIntegrated ?  '✅' : '❌'}\n` +
        `Rust 可用: ${hasRustFlag ?  '✅' : '❌'}\n` +
        `Native Reader: ${protobuf. Reader._nativeAvailable ? '✅' : '❌'}\n` +
        `Native Writer: ${protobuf. Writer._nativeAvailable ? '✅' : '❌'}`;

      this.implDetails = JSON.stringify(details, null, 2);

    } catch (error) {
      console.error('❌ Check failed:', error);
      this.statusText = `错误: ${error.message}`;
    }
  }

  performanceTest() {
    try {
      const iterations = 1000;
      const testData = {
        sessionId: "performance-test-session-id-12345",
        userPrivilege: "administrator-with-full-access",
        isTokenType: true,
        formatTimestamp: Long.fromString("9223372036854775807"),
        data: new Uint8Array(1000).fill(42) // 1KB 数据
      };

      // ========================================
      // 测试当前实现的性能
      // ========================================
      const startTime = Date.now();

      for (let i = 0; i < iterations; i++) {
        const msg = user.UserLoginResponse.create(testData);
        const encoded = user.UserLoginResponse.encode(msg).finish();
        const decoded = user.UserLoginResponse.decode(encoded);
      }

      const endTime = Date.now();
      const duration = endTime - startTime;
      const opsPerSecond = Math.round((iterations / duration) * 1000);

      const impl = protobuf.getImplementation();

      this.testResult = `性能测试结果:\n` +
        `实现:  ${impl.toUpperCase()}\n` +
        `迭代次数: ${iterations}\n` +
        `总耗时: ${duration}ms\n` +
        `性能:  ${opsPerSecond} ops/s`;

      console.log('⚡ Performance test:', this.testResult);

    } catch (error) {
      console.error('❌ Performance test failed:', error);
      this.testResult = `测试失败: ${error.message}`;
    }
  }

  build() {
    Scroll() {
      Column({ space: 15 }) {
        // 标题
        Text('Protobuf 实现检测')
          .fontSize(24)
          .fontWeight(FontWeight.Bold)
          .margin({ top: 20 })

        // 状态显示
        Text('状态信息')
          .fontSize(18)
          .fontWeight(FontWeight.Bold)
          .alignSelf(ItemAlign.Start)
          .margin({ left: 20 })

        Text(this.statusText)
          .fontSize(14)
          .fontFamily('monospace')
          .padding(15)
          .width('90%')
          .backgroundColor('#e8f5e9')
          .borderRadius(8)

        // 详细信息
        Text('详细信息')
          .fontSize(18)
          .fontWeight(FontWeight.Bold)
          .alignSelf(ItemAlign. Start)
          .margin({ left: 20 })

        Text(this.implDetails)
          .fontSize(12)
          .fontFamily('monospace')
          .padding(15)
          .width('90%')
          .backgroundColor('#f5f5f5')
          .borderRadius(8)

        // 性能测试结果
        if (this.testResult) {
          Text('性能测试')
            .fontSize(18)
            .fontWeight(FontWeight. Bold)
            .alignSelf(ItemAlign.Start)
            .margin({ left: 20 })

          Text(this.testResult)
            .fontSize(14)
            .fontFamily('monospace')
            .padding(15)
            .width('90%')
            .backgroundColor('#fff3e0')
            .borderRadius(8)
        }

        // 操作按钮
        Button('🔄 重新检测')
          .width('90%')
          .onClick(() => this.checkImplementation())

        Button('⚡ 性能测试')
          .width('90%')
          .onClick(() => this.performanceTest())

        Button('🦀 强制使用 Rust')
          .width('90%')
          .onClick(() => {
            const success = protobuf.useRust();
            this.statusText = success ?
              '✅ 已切换到 Rust 实现' :
              '❌ Rust 不可用，保持 JS 实现';
            this.checkImplementation();
          })

        Button('📦 强制使用 JS')
          .width('90%')
          .onClick(() => {
            protobuf.useJS();
            this.statusText = '✅ 已切换到 JavaScript 实现';
            this. checkImplementation();
          })

        Button('🧪 运行完整测试')
          .width('90%')
          .backgroundColor('#1976d2')
          .onClick(() => {
            this.checkImplementation();
            setTimeout(() => this.performanceTest(), 100);
          })
      }
      .width('100%')
      .padding({ bottom: 20 })
    }
    .height('100%')
    .backgroundColor('#fafafa')
  }
}