---
layout: post
title: An Inescapable Hell of Networking
tags: kubernetes, cni, networking
---

**Note:** The choices I've made are my own and are solely due to how I want to
learn more about the involved technology. Kubernetes is not the "answer to
everything", but it is the answer to "what do I think would be fun to learn?".
The most important thing, is that it WAS fun to learn. I enjoy learning, and I
enjoyed learning all the information that I'm going to talk about in this
article. In addition, the points that I make in this article are *absolutely
not* a reason to *ever* attack someone personally, and even though I
disagree with the decisions made, it is never right to contact those people
with harmful intent.

![An inescapable hell of networking, pipes, cyberpunk, vaporwave, brutalist](/assets/images/2022/09/an-inescapable-hell-of-networking-hero.png)

Image generated from Stable Diffusion: An inescapable hell of networking, pipes, cyberpunk, vaporwav, brutaliste

If you've interacted with the Cloud Native Computing Foundation, you may be
familiar with the [CNCF Landscape]. Beware - opening that link *might* crash
your computer because of how many assets it has to load. Most of these projects
have something to do with Kubernetes, container orchestration, and almost all
of them in some way interact with what we call ~~Amazon's computers~~ "The
Cloud&trade;". The fundamental design of The Cloud&trade; is that you can run
things on someone else's computer, therefore you aren't responsible for when
the computer goes offline. This is excellent, as you can typically pick and
choose your provider based on how likely it is their infrastructure is going to
[catch on fire] compared to yours.

However, this leads to what is (technically!) the source of many issues in
modern day technology: computers need to be able to talk to each other. To do
so, they need to involve components such as switches, routers, NATs, and more.
For a specific infrastructure setup (Kubernetes), this involves a component
called the Container Network Interface. It is a standardized interface for
programs that run alongside your cluster, configure routes to and from your
containers, and (most critically for this article) can be used to configure a
`NetworkPolicy` resource type. Additionally, because it is an *interface*, it
can be replaced with any component I choose to use. Therefore, we'll evaluate
and determine which of Flannel, Cilium, and Calico are ideal for setting up
networking.

A [NetworkPolicy] is a resource type in Kubernetes that can configure in [OSI]
Layer 3 (i.e. IP & Port) whether or not a resource can connect to another
resource. Therefore, it's the responsibility of the network routing component
(the CNI) to decide whether or not the resource can perform that connection.
Unfortunately, Flannel, one of the aforementioned CNI candidates, does not
support NetworkPolicy resource types. Because of that, it is no longer a valid
candidate for the Kubernetes cluster, and we have begun our descent, entering
the first level of hell.

That leaves us with two options, Cilium and Calico. The cloud provider that I'm
currently using, DigitalOcean, has a managed Kubernetes infrastructure which
came with Cilium preconfigured. This made it an ideal candidate for when I
wanted to test out CNIs in my new cluster. I tried deploying Cilium to the
cluster, but I was unable to get it working. Even after going to a DigitalOcean
managed cluster and pulling the config directly from there, I was no better
off, and we have plunged further, into the second level of hell. After that,
Calico seemed like our last chance and potential savior.

Since we're now left with Calico, I think it's a good time for a story. Over
a year before publishing this post, I was working with a company called Beeper.
I was there to help move their old Kubernetes setup (managed by [Kubesail]) to
a self-managed cluster on Hetzner. This was actually my first time setting up a
Kubernetes cluster from scratch, so I had to make the incredibly important
decision of choosing a CNI out of the gate. I decided to go with Calico because
it looked like it was the easiest to deploy. It went almost seamlessly and I
was able to deploy it with Wireguard integration, which means traffic was
encrypted (and therefore private and secure) between nodes.

Since then, Calico seems to have moved to a new form of deployment.  While
previously it was possible to just grab a manifest and throw it into a new
cluster, the system they use now is incredibly complicated. In fact, during my
first attempt at deploying Calico, I was unable to get it to successfully
deploy the fundamental resource it needs to run because the size of the file
was *too large*. After I was able to fix that issue (I used `kubectl
--server-side=true`), I was able to deploy the `Installation` resource and set
up the CNI, but a while after that, I was Spartan-kicked into the third level
of hell as the `Installation` appeared to have deleted itself and the network
became unavailable.

When I found this out, I was in a meeting with some friends, while I hoped to
get some work done. On Thursdays, some of the Hashbang community joins a video
call as we talk about some things that have happened in the past week, and I've
recently taken to doing some work while talking with my friends. This is
helpful as sometimes they can help me debug things, and in return sometimes I
will help them debug things. The first thing I did that Thursday after joining
the call was to pull up `k9s` and check out my nodes, only to see they were
reporting the `NetworkUnavailable` status. I checked the Typha (Calico)
Operator and noticed that it had lost the `Installation` resource, which meant
it could no longer deploy resources to the cluster. At this point, I decided I
would try to find a way to work around the `Installation` resource. Still on
the call, I was able to talk with a friend who was also running Calico in one
of his clusters, who mentioned that he had used the Helm templates as an
alternative to deploying the CNI. This seemed like a good plan and, given I
didn't want to be stuck in hell forever, I decided to start working on
something similar.

<!-- TODO: Deploy the CNI using the Helm charts -->

[CNCF Landscape]: https://landscape.cncf.io/
[catch on fire]: https://www.reuters.com/article/us-france-ovh-fire/-idUSKBN2B20NU
[NetworkPolicy]: https://kubernetes.io/docs/concepts/services-networking/network-policies/
[OSI]: https://en.wikipedia.org/wiki/OSI_model
[Kubesail]: https://kubesail.com/
