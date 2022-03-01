from setuptools import setup, find_packages

setup(
    name='koko_keywords',
    version='0.0.1',
    author='Kareem Kouddous',
    py_modules=['koko_keywords'],
    license='MIT',
    packages=find_packages(),
    include_package_data = True,
    install_requires=[
        'cffi==1.15.0'
    ],
    python_requires='>=3'
)
