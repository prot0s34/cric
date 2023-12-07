# cric

Container Registry Image Checker

Simple utility for check image manifest avaliability in several container registries.

### WIP

Little tool for check if image present in container registries:

 - docker.io
 - ghcr.io
 - registry.k8s.io
 - quay.io
 - gcr.io

### WIP one-liner to check: 
```
for image in library/nginx zaproxy/zaproxy:bare kube-apiserver:v1.28.2 operatorhubio/catalog kaniko-project/executor acid/postgres-operator; do
    cargo run -- "$image";
done
```

### TODO:

- refac get_token func's 
- unwrapping of matches.value_of("IMAGE") is a potential panic point if the "IMAGE" argument is somehow not provided
- parse_image_name potential error point if the image name doesn't follow the expected format
- also, latest can not present in many cases that can cause misunderstanding. Somehow provide "but other tags found" if "default" latest not found?
- also, there is some things with resolving images (as nginx -> library/nginx) that i need to understand and implement handle logic to make that app be useful
- potential panic place in  unwrap() in get_token to extract the token - need to be accurate here or make another handle solution
- Box<dyn error> provide simple signatures, but can make debugging harder cause obscures specific types of error
- no http error handling - no ability to debug network-based errors 
- maybe, i need to add second arg (like --custom-registry) to provide ability check image in specific registry (with docker/oci scheme fallback)
