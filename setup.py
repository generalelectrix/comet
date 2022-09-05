#!/usr/bin/env python
# -*- coding: utf-8 -*-
try:
    from setuptools import setup
except ImportError:
    from distutils.core import setup

requires = ['show_loop', 'python-osc', 'pyyaml',
            'argparse', 'pyenttec', 'multiprocess']

setup(
    name='COBRA_COMMANDER',
    packages=['COBRA_COMMANDER'],
    install_requires=requires,
    license='MIT',
)
