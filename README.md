# AI4Science OS TEE Worker 

## Overview

The AI4Science OS(AOS) TEE Worker is a worker sevice of AOS operator.

By registering with AOS on Operator, the worker could service the AI inference verification task.

The [Llm-In-TEE](#llm-in-tee) is a novelty framworks to run a TEE verification node service. And the AOS TEE Operators are TEE workers and building on Llm-In-TEE framwork.

## Llm-In-TEE

Run large AI models and verifiable logic clock in TEE environment.

Firstly, the semantic TEE of this repository is mainly refer to **aws nitro enclave** for now.  

Other TEE instances maybe support for later. For examples,
* Mircosoft Azure, 
* Intel SGX, 
* AMD SEV 
* or Nvidia Confidential Computing GPU

The Llm-In-TEE use the [llama.cpp](https://github.com/ggerganov/llama.cpp) as it's large AI models executor.

Second core module verifiable logic clock is an implementation of Chronos's TEE backend.   

The Chronos is a novel logical clock system designed for open networks with Byzantine participants, offering improved fault tolerance and performance. Please refer to [hetu chronos](https://github.com/hetu-project/chronos) repository for more details.

### Llm-In-TEE Arch

![architecture-diagram](./docs/img/architecture-diagram.png)

## Compile

## Build from source

```bash
git clone https://github.com/hetu-project/acl-aos-tee.git

cd acl-aos-tee

git submodule update --init --recursive
```

## Run TEE Operator

Now, this repository use the aws nitro enclave as its trust execution environment.  

So, please create a cloud virtual instance and notice choose the `Amazon-2023 linux` as base image.  
Because this base operator system is more friendly for using of the aws nitro enclave.

### Prepare Env & Configuration

1. Prepare Env & install dependency tools

```sh
cd scripts
sudo chmod +x init_env.sh
./init_env.sh
``` 

2. Download the Model

```sh
wget https://huggingface.co/TheBloke/Llama-2-7B-Chat-GGUF/resolve/main/llama-2-7b-chat.Q4_0.gguf
mv llama-2-7b-chat.Q4_0.gguf ./models
```

3. Start the TEE Environment

```sh
cd scripts
sudo chmod +x run_tee.sh
./run_tee.sh
```  

4. Start the TEE Operator

```sh
cargo build --release -p tee-worker --bin tee-worker
./target/release/tee-worker
```
