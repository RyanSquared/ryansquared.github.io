---
layout: post
title: Exploring Qubes RPC
---

In one of my previous blog posts, I discussed how I configured my system to use
qubes-rpc to enable running `gpg-agent` in a single qube, restricting access to
my Yubikey behind RPC policies. The system that implements all of this
functionality seems really neat, so in this post I'd like to break it down and
see if we can reimplement it for SSH in a way that closely resembles the UX
flow of GPG.

The first component of the split GPG system is the `gpg` wrapper, which can
emulate GPG for most use cases:

```sh
export QUBES_GPG_DOMAIN=vault
qubes-gpg-client-wrapper --list-keys
echo hi | qubes-gpg-client-wrapper --sign --armor
```

But, this setup doesn't work well for a couple commands, due to them either
not being implemented, or relying on things that would break the security model
of the system:

```sh
# This command isn't implemented
qubes-gpg-client-wrapper --card-status

# This command relies on network access
qubes-gpg-client-wrapper --recv-keys 88823A75ECAA786B0FF38B148E401478A3FBEF72

# This command moves the private key data out of the vault qube
qubes-gpg-client-wrapper --export-secret-keys
```

We can look at the source code of `qubes-gpg-client` (which I won't include for
licensing reasons, just in case) and see that it calls `qrexec-client-vm` with
a remote domain and a policy name. As an example of how `qrexec-client-vm`
works, we can run the following command:

```sh
qrexec-client-vm vault qubes.GetDate
# Prints out: 2022-09-28T07:04:20+00:00
```

We can see what runs by using `cat /etc/qubes-rpc/qubes.GetDate`, which in the
case of my vault AppVM, is `date -u -Iseconds`. It's important to note that the
command that is executed must exist in the AppVM that you're _targeting_, if
you are using a different template between your qubes, you may notice there's a
difference in the RPC commands.

### Mini Primer: AppVMs and TemplateVMs

Qubes has many types of VMs but for this blog post we're going to focus on two
types, called AppVMs and TemplateVMs. TemplateVMs have a mutable filesystem,
which means anything that we want to persist on a system level, such as a Qubes
RPC invocation command. Once a TemplateVM has been updated and has successfully
shut down, any AppVMs that use the TemplateVM can be restarted to receive the
configured changes.

### The RPC command

First, we must create a file containing the command that should be executed
when a separate VM runs `qrexec-client-vm`. At this point, we should make a
command that:

1. Determines the SSH agent socket exists
2. Signals a notification for 5 seconds that the RPC command is being executed
3. Connects the input of the command to the SSH agent socket

```sh
#!/bin/sh -eu

export SSH_AUTH_SOCK="/run/user/`id -u`/gnupg/S.gpg-agent.ssh"
test -f "$SSH_AUTH_SOCK"
notify-send -t 5000 "[$(qubesdb-read /name)] SSH access from: $QREXEC_REMOTE_DOMAIN"
socat - "UNIX-CONNECT:$SSH_AUTH_SOCK"
```

This makes the assumption that GPG has been started with SSH support and writes
its socket to the mentioned directory. This _should_ be the most common
configuration, and if using the split GPG configuration, this is most likely
how it is set up.

The executable bit should be set for the command, which can be done by using
`chmod +x /etc/qubes-rpc/qubes.Ssh`. Once this is done, the TemplateVM can be
shut down and the vault qube can be restarted to apply changes.

### Enabling the RPC policy

To allow usage of the RPC command, we must specify a policy file in the `dom0`
VM. You can do this by running the following command:

```sh
echo "@anyvm vault allow" > /etc/qubes-rpc/policy/qubes.Ssh
```

This policy will allow any VM to run the qubes.Ssh RPC command. This can be
restricted further by using `ask` instead of `allow`, or only specifying
certain AppVMs instead of any VM.

To test this, from an AppVM, we can run the following command:

```sh
echo | qrexec-client-vm vault qubes.Ssh
echo $?
```

If the output is 0, we've succeeded. If the output is 126, the request was
denied, which means there's an error with the policy file. If the output is
127, the command was not found in the vault qube.

### Testing socket functionality

Using the `socat` command, we can create a relay from a UNIX socket (like
ssh-agent expects) to a command:

```sh
export SSH_AUTH_SOCK="/run/user/`id -u`/ssh/S.ssh-agent"
export QUBES_SSH_DOMAIN="${QUBES_GPG_DOMAIN:-vault}"
mkdir -p "$(dirname $SSH_AUTH_SOCK)"
socat "UNIX-LISTEN:$SSH_AUTH_SOCK,fork" "EXEC:qrexec-client-vm $QUBES_SSH_DOMAIN qubes.Ssh"
```

From another terminal in the same AppVM, we can run a command to print all
public keys that the SSH agent has a matching private key for:

```sh
export SSH_AUTH_SOCK="/run/user/`id -u`/ssh/S.ssh-agent"
ssh-add -L
echo $?
```

If the exit status of the command is 0, we've succeeded. If the output is 2,
the socket wasn't correctly created or can't be found by the agent.


### Forwarding a qube's SSH agent

Using systemd's socket functionality, we can create a service file that will
automatically be created every time a client connects to the socket by using
`Accept=true` in the socket file and `StandardInput=socket` in the service
template file. The "@" in the service file is significant; this is what signals
systemd to run the command for every new client.

These files must be created in the TemplateVMs that you wish to grant access to
the SSH agent.

```systemd
# /etc/systemd/user/ssh-agent.socket
[Unit]
Description=Forward connections to an SSH agent to a remote Qube

[Socket]
ListenStream=%t/ssh/S.ssh-agent
SocketMode=0600
DirectoryMode=0700
Accept=true

[Install]
WantedBy=sockets.target
```

```systemd
# /etc/systemd/user/ssh-agent@.service
[Unit]
Description=Forward connections to an SSH agent to a remote Qube

[Service]
ExecStart=qrexec-client-vm vault qubes.Ssh
StandardInput=socket
```

Once the unit files are created, we can enable the socket to automatically
start; once a qube has started, it will automatically load the socket unit and
create the socket file.

```sh
sudo ln -s /etc/systemd/user/ssh-agent.socket /etc/systemd/user/sockets.target.wants/
```

The following can also be placed in `.bashrc`, `.zshrc`, or similar files to
automatically configure the SSH agent:

```sh
export SSH_AUTH_SOCK="${XDG_RUNTIME_DIR:-/run/user/`id -u`}/ssh/S.ssh-agent"
```

### Improving the User Experience

The split GPG setup has a convenient section of code that can automatically
prompt whether or not a qube can access the SSH qube. While I'm not going to
post that code snippet here (for licensing reasons), it can be copied almost
entirely without modification for your own uses. Figuring out how to make the
`/var/run` directory writable is an exercise left to the reader, because I'm
tired and would be completely fine with sharing auth sessions between SSH and
GPG if it means I can go to bed.
