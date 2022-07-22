pub fn execute_commands(&cmd: IndexMap<usize, PropChanges>) {
    cmd.write()
        .unwrap()
        .par_iter()
        .for_each_with(done_commands.clone(), |a, (id, command)| match command {
            PropChanges::ChangePoint(command) => {
                let (f, done) = command
                    .animation_id
                    .map(|id| {
                        self.animations_storage
                            .map
                            .read()
                            .unwrap()
                            .get(&id)
                            .map(|AnimationState(_, value, done)| (*value, *done))
                            .unwrap_or((1.0, true))
                    })
                    .unwrap_or((1.0, true));

                *command.change.target.value.write().unwrap() =
                    interpolate(command.change.from, command.change.to, f);
                if command.needs_repaint {
                    command.target_needs_repaint.store(true, Ordering::Relaxed);
                }

                if done {
                    a.write().unwrap().push(*id);
                }
            }
            PropChanges::ChangeF64(command) => {
                let (f, done) = command
                    .animation_id
                    .map(|id| {
                        self.animations_storage
                            .map
                            .read()
                            .unwrap()
                            .get(&id)
                            .map(|AnimationState(_, value, done)| (*value, *done))
                            .unwrap_or((1.0, true))
                    })
                    .unwrap_or((1.0, true));

                *command.change.target.value.write().unwrap() =
                    interpolate(command.change.from, command.change.to, f);

                if command.needs_repaint {
                    command.target_needs_repaint.store(true, Ordering::Relaxed);
                }

                if done {
                    a.write().unwrap().push(*id);
                }
            }
            PropChanges::ChangeBorderRadius(command) => {
                let (f, done) = command
                    .animation_id
                    .map(|id| {
                        self.animations_storage
                            .map
                            .read()
                            .unwrap()
                            .get(&id)
                            .map(|AnimationState(_, value, done)| (*value, *done))
                            .unwrap_or((1.0, true))
                    })
                    .unwrap_or((1.0, true));

                *command.change.target.value.write().unwrap() =
                    interpolate(command.change.from, command.change.to, f);

                if command.needs_repaint {
                    command.target_needs_repaint.store(true, Ordering::Relaxed);
                }

                if done {
                    a.write().unwrap().push(*id);
                }
            }
            PropChanges::ChangePaintColor(command) => {
                let (f, done) = command
                    .animation_id
                    .map(|id| {
                        self.animations_storage
                            .map
                            .read()
                            .unwrap()
                            .get(&id)
                            .map(|AnimationState(_, value, done)| (*value, *done))
                            .unwrap_or((1.0, true))
                    })
                    .unwrap_or((1.0, true));

                *command.change.target.value.write().unwrap() =
                    interpolate(command.change.from.clone(), command.change.to.clone(), f);

                if command.needs_repaint {
                    command.target_needs_repaint.store(true, Ordering::Relaxed);
                }

                if done {
                    a.write().unwrap().push(*id);
                }
            }
        });
}
